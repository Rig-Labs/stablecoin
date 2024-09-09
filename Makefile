# This imports the environment variables from the .env file
include .env
export $(shell sed 's/=.*//' .env)
############################# HELP MESSAGE #############################
# Make sure the help command stays first, so that it's printed by default when `make` is called without arguments
.PHONY: help tests build-and-test generate-types deploy
help:
	@grep -E '^[a-zA-Z0-9_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

-----------------------------: ## 

build-and-test: ## Build and run tests
	@sh scripts/build-and-test.sh

generate-types: ## Generate types
	@sh scripts/generate-types.sh

deploy: ## Run the deployment script for core contracts
	@cd deploy-scripts && RPC=$(RPC) SECRET=$(SECRET) cargo run deploy

add-assets: ## Run the script to add assets to the protocol
	@cd deploy-scripts && RPC=$(RPC) SECRET=$(SECRET) cargo run add-assets
