#!/usr/bin/env bash

set -e

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <demo name>"
  exit 2;
fi

DEMO_NAME=$1;

PROJECT_DIR=$(git rev-parse --show-toplevel)
DEMO_DIR="${PROJECT_DIR}/create/new_project/demos/${DEMO_NAME}"

if ! [ -d "$DEMO_DIR" ]; then
  echo "Demo dir does not exit: $DEMO_DIR";
  exit 2;
fi

LOG_FILE=$(mktemp)

## Start tests

cd "${DEMO_DIR}"
yarn install > /dev/null

## Start-up environment
yarn run start-env > /dev/null &
STARTENV_PID=$!
ENV_READY=false

CCA_TIMEOUT=60

function check_containers() {
  ERROR=false
  for CONTAINER in ethereum bitcoin cnd_0 cnd_1; do
    NUM=$(docker ps -qf name=${CONTAINER} |wc -l)
    if test "$NUM" -ne 1; then
      ERROR=true;
      break;
    fi
  done
  $ERROR && echo 1 || echo 0
}

while [ $CCA_TIMEOUT -gt 0 ]; do
    if [ "$(check_containers)" -eq 0 ]; then
      CCA_TIMEOUT=0
      ENV_READY=true
    else
      sleep 1;
      CCA_TIMEOUT=$((CCA_TIMEOUT-1));
    fi
done

if ! $ENV_READY; then
  echo "FAIL: ${CONTAINER} docker container was not started."
  kill $STARTENV_PID;
  wait $STARTENV_PID;
  exit 1;
fi

# Run the example
RUN_TIMEOUT=60
TEST_PASSED=false

NON_INTERACTIVE=true yarn run swap > "${LOG_FILE}" 2>&1 &
RUN_PID=$!

function check_swap() {
  local LOG_FILE=$1;
  grep -q "Swapped!" "$LOG_FILE";
  echo $?;
}

while [ $RUN_TIMEOUT -gt 0 ]; do
  if [ "$(check_swap "$LOG_FILE")" -eq 0 ]; then
    RUN_TIMEOUT=0;
    TEST_PASSED=true;
  else
    sleep 1;
    RUN_TIMEOUT=$((RUN_TIMEOUT-1));
  fi
done

if $TEST_PASSED; then
  echo "SUCCESS: It swapped.";
  EXIT_CODE=0;
else
  echo "FAIL: It did not swap.";
  cat "$LOG_FILE";
  EXIT_CODE=1;
fi

wait $RUN_PID || true;

kill -s SIGINT $STARTENV_PID;
wait $STARTENV_PID || true;

# Ensure clean up
yarn run clean-env > /dev/null &

# Count the number of containers still running
function check_containers() {
  ERROR=false
  for CONTAINER in ethereum bitcoin cnd_0 cnd_1; do
    NUM=$(docker ps -qf name=${CONTAINER} |wc -l)
    if test "$NUM" -eq 1; then
      ERROR=true;
      break;
    fi
  done
  $ERROR && echo 1 || echo 0
}

# Wait for cleaning up environment
TIMEOUT=10
while [ $TIMEOUT -gt 0 ]; do
    if [ "$(check_containers)" -eq 0 ]; then
      TEST_PASSED=true;
      TIMEOUT=0
    else
      echo "Waiting for containers to die";
      sleep 1;
      TIMEOUT=$((TIMEOUT-1));
    fi
done

rm -f "${LOG_FILE}"

exit $EXIT_CODE;
