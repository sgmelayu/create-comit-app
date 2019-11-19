use crate::cnd_settings;
use crate::docker::bitcoin::{self, BitcoinNode};
use crate::docker::ethereum::{self, EthereumNode};
use crate::docker::Cnd;
use crate::docker::{create_network, Node};
use crate::env::temp_fs;
use crate::env::Error;
use crate::print_progress;
use envfile::EnvFile;
use futures::{FutureExt, TryFutureExt};
use rand::{thread_rng, Rng};
use rust_bitcoin::util::bip32::ChildNumber;
use rust_bitcoin::util::bip32::ExtendedPrivKey;
use rust_bitcoin::Amount;
use rust_bitcoin::PrivateKey;
use secp256k1::{Secp256k1, SecretKey};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::prelude::stream;
use tokio::prelude::{Future, Stream};
use tokio::runtime::Runtime;
use web3::types::{Address, U256};

pub struct Services {
    pub docker_network_id: String,
    pub bitcoin_node: Arc<Node<BitcoinNode>>,
    pub ethereum_node: Arc<Node<EthereumNode>>,
    pub cnds: Arc<Vec<Node<Cnd>>>,
}

#[derive(Clone)]
pub struct BitcoinAccount {
    master_extended_private_key: ExtendedPrivKey,
    derived_private_key: PrivateKey,
    derived_address: rust_bitcoin::Address,
}

#[derive(Clone)]
pub struct EthereumAccount {
    private_key: SecretKey,
}

fn build_futures() -> Result<
    (
        Vec<BitcoinAccount>,
        Vec<EthereumAccount>,
        PathBuf,
        impl Future<Item = String, Error = ()>,
        impl Future<Item = Node<BitcoinNode>, Error = ()>,
        impl Future<Item = (Address, Node<EthereumNode>), Error = ()>,
        impl Future<Item = Vec<Node<Cnd>>, Error = ()>,
    ),
    Error,
> {
    let bitcoin_accounts = vec![new_bitcoin_account()?, new_bitcoin_account()?];

    let ethereum_accounts = vec![new_ethereum_account(), new_ethereum_account()];

    let env_file_path = temp_fs::env_file_path()?;
    let docker_network_create = create_network().map_err(|e| {
        eprintln!("Issue creating Docker network: {:?}", e);
    });
    let bitcoin_node = start_bitcoin_node(&env_file_path, bitcoin_accounts.clone()).map_err(|e| {
        eprintln!("Issue starting Bitcoin node: {:?}", e);
    });
    let ethereum_node =
        start_ethereum_node(&env_file_path, ethereum_accounts.clone()).map_err(|e| {
            eprintln!("Issue starting Ethereum node: {:?}", e);
        });
    let cnds = {
        let (path, string) = temp_fs::temp_folder()?;
        start_cnds(&env_file_path, path, string).map_err(|e| {
            eprintln!("Issue starting cnds: {:?}", e);
        })
    };
    Ok((
        bitcoin_accounts,
        ethereum_accounts,
        env_file_path,
        docker_network_create,
        bitcoin_node,
        ethereum_node,
        cnds,
    ))
}

pub fn execute(runtime: &mut Runtime, terminate: &Arc<AtomicBool>) -> Result<Services, Error> {
    let (
        bitcoin_accounts,
        ethereum_accounts,
        env_file_path,
        docker_network_create,
        bitcoin_node,
        ethereum_node,
        cnds,
    ) = build_futures()?;

    let env_file_str = temp_fs::create_env_file()?;

    print_progress!("Creating Docker network (create-comit-app)");
    let docker_network_id = runtime.block_on(docker_network_create).map_err(|e| {
        eprintln!("Could not create docker network, aborting...\n{:?}", e);
    })?;
    println!("✓");
    check_signal(terminate)?;

    print_progress!("Starting Ethereum node");
    let (contract_address, ethereum_node) = runtime
        .block_on(ethereum_node)
        .map_err(|e| {
            eprintln!("Could not start Ethereum node, aborting...\n{:?}", e);
        })
        .map(|(contract_address, node)| (contract_address, Arc::new(node)))?;
    println!("✓");
    check_signal(terminate)?;

    print_progress!("Starting Bitcoin node");
    let bitcoin_node = runtime
        .block_on(bitcoin_node)
        .map_err(|e| {
            eprintln!("Could not start bitcoin node, aborting...\n{:?}", e);
        })
        .map(Arc::new)?;
    println!("✓");
    check_signal(terminate)?;

    print_progress!("Writing configuration in env file");
    let mut envfile = EnvFile::new(env_file_path.clone()).map_err(|e| {
        eprintln!("Could not read {} file, aborting...\n{:?}", env_file_str, e);
    })?;

    for (i, account) in bitcoin_accounts.iter().enumerate() {
        envfile.update(
            format!("BITCOIN_HD_KEY_{}", i).as_str(),
            format!("{}", account.master_extended_private_key).as_str(),
        );
    }

    for (i, account) in ethereum_accounts.iter().enumerate() {
        envfile.update(
            format!("ETHEREUM_KEY_{}", i).as_str(),
            format!("{}", account.private_key).as_str(),
        );
    }

    envfile.update(
        "ERC20_CONTRACT_ADDRESS",
        format!("0x{:x}", contract_address).as_str(),
    );

    envfile.write().map_err(|e| {
        eprintln!(
            "Could not write {} file, aborting...\n{:?}",
            env_file_str, e
        );
    })?;
    println!("✓");
    check_signal(terminate)?;

    print_progress!("Starting two cnds");
    let cnds = runtime
        .block_on(cnds)
        .map_err(|e| {
            eprintln!("Could not start cnds, cleaning up...\n{:?}", e);
        })
        .map(Arc::new)?;
    println!("✓");
    check_signal(terminate)?;

    println!("🎉 Environment is ready, time to create a COMIT app!");
    Ok(Services {
        docker_network_id,
        bitcoin_node,
        ethereum_node,
        cnds,
    })
}

fn start_bitcoin_node(
    envfile_path: &PathBuf,
    bitcoin_accounts: Vec<BitcoinAccount>,
) -> impl Future<Item = Node<BitcoinNode>, Error = Error> {
    Node::<BitcoinNode>::start(envfile_path.clone(), "bitcoin")
        .map_err(Error::Docker)
        .and_then(move |node| {
            stream::iter_ok(bitcoin_accounts).fold(node, |node, account| {
                bitcoin::fund(
                    node.node_image.username.clone(),
                    node.node_image.password.clone(),
                    node.node_image.endpoint.clone(),
                    bitcoin::derive_address(account.derived_private_key.key),
                    Amount::from_sat(1_000_000_000),
                )
                .map_err(Error::BitcoinFunding)
                .map(|_| node)
            })
        })
}

fn start_ethereum_node(
    envfile_path: &PathBuf,
    ethereum_accounts: Vec<EthereumAccount>,
) -> impl Future<Item = (Address, Node<EthereumNode>), Error = Error> {
    Node::<EthereumNode>::start(envfile_path.clone(), "ethereum")
        .map_err(Error::Docker)
        .and_then({
            let secret_keys = ethereum_accounts
                .clone()
                .into_iter()
                .map(|account| account.private_key)
                .collect();
            |node| {
                let web3 = node.node_image.http_client.clone();
                ethereum::fund_ether(web3, secret_keys, U256::from(100u128 * 10u128.pow(18)))
                    .boxed()
                    .compat()
                    .map_err(Error::EtherFunding)
                    .map(|_| node)
            }
        })
        .and_then({
            let secret_keys = ethereum_accounts
                .into_iter()
                .map(|account| account.private_key)
                .collect();
            |node| {
                let web3 = node.node_image.http_client.clone();
                ethereum::fund_erc20(web3, secret_keys, U256::from(100u128 * 10u128.pow(18)))
                    .boxed()
                    .compat()
                    .map_err(Error::EtherFunding)
                    .map(|contract_address| (contract_address, node))
            }
        })
}

fn start_cnds(
    envfile_path: &PathBuf,
    config_folder: PathBuf,
    config_folder_str: String,
) -> impl Future<Item = Vec<Node<Cnd>>, Error = Error> {
    stream::iter_ok(vec![0, 1])
        .and_then({
            let envfile_path = envfile_path.clone();

            move |i| {
                tokio::fs::create_dir_all(config_folder.clone())
                    .map_err(Error::CreateTmpFiles)
                    .and_then({
                        let config_folder = config_folder.clone();

                        move |_| {
                            let settings = cnd_settings::Settings {
                                bitcoin: cnd_settings::Bitcoin {
                                    network: String::from("regtest"),
                                    node_url: "http://bitcoin:18443".to_string(),
                                },
                                ethereum: cnd_settings::Ethereum {
                                    network: String::from("regtest"),
                                    node_url: "http://ethereum:8545".to_string(),
                                },
                                ..Default::default()
                            };

                            let config_file = config_folder.join("cnd.toml");
                            let settings = toml::to_string(&settings)
                                .expect("could not serialize hardcoded settings");

                            tokio::fs::write(config_file, settings).map_err(Error::WriteConfig)
                        }
                    })
                    .and_then({
                        let envfile_path = envfile_path.clone();
                        let config_folder_str = config_folder_str.clone();
                        move |_| {
                            let volume = format!("{}:/config", config_folder_str);

                            Node::<Cnd>::start_with_volume(
                                envfile_path.to_path_buf(),
                                format!("cnd_{}", i).as_str(),
                                &volume,
                            )
                            .map_err(Error::Docker)
                        }
                    })
            }
        })
        .collect()
}

fn new_bitcoin_account() -> Result<BitcoinAccount, Error> {
    // generate master key (hd key)
    let hd_key = ExtendedPrivKey::new_master(rust_bitcoin::Network::Regtest, &{
        let mut seed = [0u8; 32];
        thread_rng().fill_bytes(&mut seed);

        seed
    })?;

    // define derivation path to derive private keys from the master key
    let derivation_path = vec![
        ChildNumber::from_hardened_idx(44)?,
        ChildNumber::from_hardened_idx(1)?,
        ChildNumber::from_hardened_idx(0)?,
        ChildNumber::from_normal_idx(0)?,
        ChildNumber::from_normal_idx(0)?,
    ];

    // derive a private key from the master key
    let priv_key = hd_key
        .derive_priv(&Secp256k1::new(), &derivation_path)
        .map(|secret_key| secret_key.private_key)?;

    // derive an address from the private key
    let address = bitcoin::derive_address(priv_key.key);

    Ok(BitcoinAccount {
        master_extended_private_key: hd_key,
        derived_private_key: priv_key,
        derived_address: address,
    })
}

fn new_ethereum_account() -> EthereumAccount {
    EthereumAccount {
        private_key: SecretKey::new(&mut thread_rng()),
    }
}

fn check_signal(terminate: &Arc<AtomicBool>) -> Result<(), Error> {
    if terminate.load(Ordering::Relaxed) {
        Err(Error::SignalReceived)
    } else {
        Ok(())
    }
}
