import axios from "axios";
import { makeArchiveName } from "common";
import * as fs from "fs";
import path from "path";
import * as targz from "targz";
import util from "util";

const extract = util.promisify(targz.decompress);

export default async function download(
  version: string,
  binTarget: string
): Promise<void> {
  const targetDir = path.dirname(binTarget);

  const archiveName = makeArchiveName("create-comit-app", version);
  const archivePath = targetDir + "/" + archiveName;

  if (!fs.existsSync(targetDir)) {
    fs.mkdirSync(targetDir, { recursive: true });
  }

  if (fs.existsSync(archivePath)) {
    fs.unlinkSync(archivePath);
  }

  const url = `https://github.com/comit-network/create-comit-app/releases/download/create-comit-app-${version}/${archiveName}`;

  let response = await axios({
    url,
    method: "GET",
    responseType: "stream",
  });

  if (response.status === 302) {
    response = await axios({
      url: response.headers.location,
      method: "GET",
      responseType: "stream",
    });
  }

  const file = fs.createWriteStream(archivePath);

  response.data.pipe(file);
  await new Promise((resolve, reject) => {
    response.data.on("end", () => {
      resolve();
    });

    response.data.on("error", (err: any) => {
      reject(err);
    });
  }).catch();

  await extract({
    src: archivePath,
    dest: targetDir,
  });
  fs.unlinkSync(archivePath);
  fs.renameSync(targetDir + "/create-comit-app", binTarget);
  fs.chmodSync(binTarget, 755);
}
