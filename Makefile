.DEFAULT_GOAL := help

# .PHONY targets are not associated with any particular file; ensures these commands always run when called
.PHONY: help

# Display help information
help: ## Show possible targets
	@echo "Available targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'

deploy: ## Deploy the application on production
	@echo "Deploying the application on production"
	@read -p "Are you sure you want to deploy the application on production? [y/N] " answer; \
	if [ "$$answer" != "y" ]; then \
		echo "Deployment cancelled"; \
		exit 1; \
	fi
	# deploy the application
	@echo "Deploying the application on production..."
	@ansible-playbook ansible/deploy.yaml -i ansible/inventory.yaml
