# ========== Configuration ==========
ZAPHENATH_REPO ?= ../zaphenath ABI_SRC_PATH = abi/Zaphenath.json
ABI_REMOTE_URL=https://raw.githubusercontent.com/Astervia/zaphenath/main/abi/Zaphenath.json
ABI_SRC_PATH ?= ./abi/Zaphenath.json

ANVIL_RPC_URL=http://localhost:8545
ANVIL_PORT=8545
ANVIL_LOG=.anvil.log

# ------------------------------------------------------------------
# 🔑  DEV KEYS PRODUCED BY THE DEFAULT HARDHAT / FOUNDRY MNEMONIC
#
# index | address (20 bytes)                         | private key (32 bytes)
# ------|--------------------------------------------|--------------------------------------------------------------------
#   0   | 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 | 0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
# ------------------------------------------------------------------------------------------------------------------------

# -- Key to be used by the test code itself -----------------------------------
TEST_PRIVKEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80
TEST_ADDR=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266
TEST_PRIVKEY_PATH=./testkey.hex

# ========== Targets ==========

help: ## Show this help
	@echo "📘 Available Make commands:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-22s\033[0m %s\n", $$1, $$2}'

test-local: ## Deploy contract, set up test config, and run tests against local Anvil
	@set -e; \
	echo "🚀 Launching local testnet with Anvil…"; \
	echo "$(TEST_PRIVKEY)" > $(TEST_PRIVKEY_PATH); \
	anvil --port $(ANVIL_PORT) \
	      --mnemonic "test test test test test test test test test test test junk" \
	      --block-time 0.1 \
	      > $(ANVIL_LOG) 2>&1 & \
	ANVIL_PID=$$!; \
	trap 'echo "🛑 Killing Anvil (PID $$ANVIL_PID)"; kill $$ANVIL_PID; rm ./testkey.hex' EXIT; \
	\
	i=0; until cast block-number --rpc-url $(ANVIL_RPC_URL) > /dev/null 2>&1; do \
	  i=$$((i+1)); \
	  if [ $$i -ge 20 ]; then \
	    echo "❌ Anvil did not start within expected time"; \
	    exit 1; \
	  fi; \
	  sleep 0.5; \
	done; \
	echo "💰 Account balance:"; \
	cast balance ${TEST_ADDR} --rpc-url ${ANVIL_RPC_URL}; \
	\
	echo "💽 Deploying Zaphenath contract…"; \
	( \
	  cd $(ZAPHENATH_REPO) && \
	  forge script script/Zaphenath.s.sol --broadcast --private-key $(TEST_PRIVKEY) --rpc-url $(ANVIL_RPC_URL) \
	) > /dev/null; \
	\
	CONTRACT_ADDRESS=$$(jq -r '.transactions[] | select(.contractName=="Zaphenath") | .contractAddress' $(ZAPHENATH_REPO)/broadcast/Zaphenath.s.sol/31337/run-latest.json); \
	echo "✅ Contract deployed at: $$CONTRACT_ADDRESS"; \
	\
	TEMP_CONFIG=$$(mktemp); \
	echo "[{\"key_id\":\"testkey123\",\"contract_address\":\"$$CONTRACT_ADDRESS\",\"private_key_path\":\"$(TEST_PRIVKEY_PATH)\"}]" > $$TEMP_CONFIG; \
	\
	echo "🧪 Running tests…"; \
	ZAPHENATH_TEST_MOCK=false \
	ZAPHENATH_TEST_RPC=$(ANVIL_RPC_URL) \
	ZAPHENATH_TEST_CONTRACT=$$CONTRACT_ADDRESS \
	ZAPHENATH_TEST_PRIVKEY=$(TEST_PRIVKEY_PATH) \
	ZAPHENATH_TEST_CONFIG=$$TEMP_CONFIG \
	cargo test $(EXTRA_ARGS)

fetch-abi: ## Download latest ABI from the remote Zaphenath contract repo
	@echo "⬇️  Fetching ABI from $(ABI_REMOTE_URL)…"
	@curl -sSL $(ABI_REMOTE_URL) -o $(ABI_SRC_PATH)
	@echo "✅ ABI written to $(ABI_SRC_PATH)"
