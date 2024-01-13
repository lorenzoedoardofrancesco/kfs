IMAGE_NAME = alpinerustimage
VOLUME_NAME = alpinerustvolume
CONTAINER_NAME = alpinerust
CHECKSUM_FILE = .checksum
FILES_CHANGED_FLAG = .files_changed
ISO_FILE = kfs.iso

NO_OUTPUT = > /dev/null 2>&1
CHECKMARK = $(GREEN)✓$(WHITE)

SRC_FILES = Cargo.toml i386-unknown-none.json linker.ld Makefile_docker

SRC_DIRS = src isofiles .cargo

YELLOW = \033[1;33m
GREEN = \033[1;32m
RED = \033[1;31m
WHITE = \033[0;37m

all: docker-build docker-create docker-start check-checksums 

docker-volume:
	@if [ -z "$$(docker volume ls -q -f name=$(VOLUME_NAME))" ]; then \
		echo "$(YELLOW)Creating Docker volume $(VOLUME_NAME)...$(WHITE)"; \
		docker volume create $(VOLUME_NAME) > /dev/null 2>&1; \
		echo "$(GREEN)Docker volume $(VOLUME_NAME) created successfully.$(WHITE)"; \
	else \
		echo "$(GREEN)Docker volume $(VOLUME_NAME) already exists.$(WHITE)"; \
	fi

docker-build:
	@if [ -z "$$(docker images -q $(IMAGE_NAME))" ]; then \
		echo "$(YELLOW)Building Docker image $(IMAGE_NAME)...$(WHITE)"; \
		docker build -t $(IMAGE_NAME) . > /dev/null 2>&1; \
		echo "$(GREEN)Docker image $(IMAGE_NAME) built successfully.$(WHITE)"; \
	else \
		echo "$(GREEN)Docker image $(IMAGE_NAME) already exists.$(WHITE)"; \
	fi

docker-create: docker-volume
	@if [ -z "$$(docker ps -aq -f name=^$(CONTAINER_NAME)$$)" ]; then \
		echo "$(YELLOW)Creating Docker container $(CONTAINER_NAME)...$(WHITE)"; \
		docker create --name $(CONTAINER_NAME) -v $(VOLUME_NAME):/kfs $(IMAGE_NAME) > /dev/null 2>&1; \
		echo "$(GREEN)Docker container $(CONTAINER_NAME) created successfully.$(WHITE)"; \
	else \
		echo "$(GREEN)Docker container $(CONTAINER_NAME) already exists.$(WHITE)"; \
	fi

docker-start:
	@if [ -z "$$(docker ps -q -f name=^$(CONTAINER_NAME)$$ -f status=running)" ]; then \
		echo "$(YELLOW)Starting Docker container $(CONTAINER_NAME)...$(WHITE)"; \
		docker start $(CONTAINER_NAME) > /dev/null 2>&1; \
		echo "$(GREEN)Docker container $(CONTAINER_NAME) started successfully.$(WHITE)"; \
	else \
		echo "$(GREEN)Docker container $(CONTAINER_NAME) is already running.$(WHITE)"; \
	fi

transfer-and-build:
	$(eval MOUNTPOINT=$(shell docker volume inspect --format '{{ .Mountpoint }}' $(VOLUME_NAME)))
	@cp -r .cargo $(MOUNTPOINT)/
	@cp -r isofiles $(MOUNTPOINT)/
	@cp -r src $(MOUNTPOINT)/
	@cp Cargo.toml $(MOUNTPOINT)/
	@cp i386-unknown-none.json $(MOUNTPOINT)/
	@cp linker.ld $(MOUNTPOINT)/
	@cp Makefile_docker $(MOUNTPOINT)/Makefile
	@echo "$(YELLOW)\n--- Building KFS ---\n$(WHITE)"
	@if ! docker exec -t $(CONTAINER_NAME) make; then \
		echo "$(RED)\n--- Error in building KFS ---\n$(WHITE)"; \
		rm -f $(CHECKSUM_FILE); \
		exit 1; \
	fi
	@echo "$(GREEN)\n--- Build finished ---\n$(WHITE)"
	@cp $(MOUNTPOINT)/$(ISO_FILE) $(ISO_FILE)

check-checksums:
	@echo "\nChecking for file changes..."
	@{ \
	find $(SRC_DIRS) -type f -exec md5sum {} +; \
	md5sum $(SRC_FILES); \
	} | sort | md5sum > $(CHECKSUM_FILE).new
	@if [ ! -f $(CHECKSUM_FILE) ] || ! cmp -s $(CHECKSUM_FILE) $(CHECKSUM_FILE).new; then \
		echo "Files have changed, executing build..."; \
		mv $(CHECKSUM_FILE).new $(CHECKSUM_FILE); \
		$(MAKE) transfer-and-build; \
	else \
		echo "No changes in files, skipping build."; \
		rm $(CHECKSUM_FILE).new; \
	fi

run: all
	@if [ -f $(ISO_FILE) ]; then \
		qemu-system-i386 -drive file=kfs.iso,format=raw,index=0,media=disk -m 32 -serial file:output.log -serial stdio -display curses; \
	else \
		echo "No $(ISO_FILE) found, please run 'make' first."; \
	fi

debug: all
	@if [ -f $(ISO_FILE) ]; then \
		qemu-system-i386 -drive file=kfs.iso,format=raw,index=0,media=disk -s -S -m 32 -serial file:output.log -serial stdio -display curses; \
	else \
		echo "No $(ISO_FILE) found, please run 'make' first."; \
	fi

doc: all
	$(eval MOUNTPOINT=$(shell docker volume inspect --format '{{ .Mountpoint }}' $(VOLUME_NAME)))
	@docker exec -t $(CONTAINER_NAME) cargo doc --document-private-items
	@echo "$(YELLOW)\n--- Copying documentation from Docker volume ---\n$(WHITE)"
	@cp $(MOUNTPOINT)/target/i386-unknown-none/doc doc -r
	@echo "$(GREEN)Documentation copied to doc$(WHITE)"
	@open doc/kfs/index.html

cargo-clean:
	@docker exec -t $(CONTAINER_NAME) cargo clean

clean:
	@if [ ! -z "$$(docker ps -aq -f name=^$(CONTAINER_NAME)$$)" ]; then \
		docker stop $(CONTAINER_NAME); \
		docker rm $(CONTAINER_NAME); \
		docker rmi $(IMAGE_NAME); \
	else \
		echo "No such container: $(CONTAINER_NAME)"; \
	fi
	@if [ ! -z "$$(docker volume ls -q -f name=^$(VOLUME_NAME)$$)" ]; then \
		echo "Deleting Docker volume $(VOLUME_NAME)..."; \
		docker volume rm $(VOLUME_NAME); \
	else \
		echo "No such volume: $(VOLUME_NAME)"; \
	fi
	rm -f $(CHECKSUM_FILE)
	rm -f $(FILES_CHANGED_FLAG)
	rm -f $(ISO_FILE)
	rm -f output.log
	rm -f Cargo.lock
	rm -rf doc
	rm -rf target

fclean: clean
	@if [ ! -z "$$(docker images -q $(IMAGE_NAME))" ]; then \
		docker rmi -f $(IMAGE_NAME); \
	else \
		echo "No such image: $(IMAGE_NAME)"; \
	fi

.PHONY: all docker-build docker-create docker-start transfer-and-build check-checksums doc clean fclean
