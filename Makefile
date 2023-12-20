IMAGE_NAME = alpinerustimage
VOLUME_NAME = alpinerustvolume
CONTAINER_NAME = alpinerust
CHECKSUM_FILE = .checksums
FILES_CHANGED_FLAG = .files_changed

NO_OUTPUT = > /dev/null 2>&1
CHECKMARK = $(GREEN)✓$(WHITE)

SRC_FILES = Cargo.toml i386-unknown-none.json linker.ld Makefile_docker

SRC_DIRS = src isofiles .cargo

YELLOW = \033[0;33m
GREEN = \033[0;32m
WHITE = \033[0;37m

all: docker-build docker-create docker-start check-checksums

docker-volume:
	@if [ -z "$$(docker volume ls -q -f name=$(VOLUME_NAME))" ]; then \
		echo "Creating Docker volume $(VOLUME_NAME)..."; \
		docker volume create $(VOLUME_NAME); \
	else \
		echo "$(CHECKMARK) Docker volume $(VOLUME_NAME) already exists."; \
	fi

docker-build:
	@if [ -z "$$(docker images -q $(IMAGE_NAME))" ]; then \
		echo "Building Docker image $(IMAGE_NAME)..."; \
		docker build -t $(IMAGE_NAME) .; \
	else \
		echo "$(CHECKMARK) Docker image $(IMAGE_NAME) already exists."; \
	fi

docker-create: docker-volume
	@if [ -z "$$(docker ps -aq -f name=^$(CONTAINER_NAME)$$)" ]; then \
		echo "Creating Docker container $(CONTAINER_NAME)..."; \
		docker create --name $(CONTAINER_NAME) -v $(VOLUME_NAME):/kfs $(IMAGE_NAME); \
	else \
		echo "$(CHECKMARK) Docker container $(CONTAINER_NAME) already exists."; \
	fi

docker-start:
	@if [ -z "$$(docker ps -q -f name=^$(CONTAINER_NAME)$$ -f status=running)" ]; then \
		echo "Starting Docker container $(CONTAINER_NAME)..."; \
		docker start $(CONTAINER_NAME); \
	else \
		echo "$(CHECKMARK) Docker container $(CONTAINER_NAME) is already running."; \
	fi

transfer-and-build: check-checksums
	$(eval MOUNTPOINT=$(shell docker volume inspect --format '{{ .Mountpoint }}' $(VOLUME_NAME)))
	cp -r .cargo $(MOUNTPOINT)/
	cp -r isofiles $(MOUNTPOINT)/
	cp -r src $(MOUNTPOINT)/
	cp Cargo.toml $(MOUNTPOINT)/
	cp i386-unknown-none.json $(MOUNTPOINT)/
	cp linker.ld $(MOUNTPOINT)/
	cp Makefile_docker $(MOUNTPOINT)/Makefile
	echo "$(YELLOW)\n--- Building KFS ---\n$(WHITE)"
	docker exec -t $(CONTAINER_NAME) make
	echo "$(GREEN)\n--- Build finished ---\n$(WHITE)"
	cp $(MOUNTPOINT)/kfs.iso kfs.iso
	$(MAKE) update-checksums

check-checksums:
	@echo "Checking for file changes..."
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

run:
	@if [ -f kfs.iso ]; then \
		qemu-system-i386 kfs.iso -m 256 -serial file:output.log; \
	else \
		echo "No kfs.iso found, please run 'make' first."; \
	fi

doc:
	@cargo doc --open --document-private-items

clean:
	@if [ ! -z "$$(docker ps -aq -f name=^$(CONTAINER_NAME)$$)" ]; then \
		docker stop $(CONTAINER_NAME); \
		docker rm $(CONTAINER_NAME); \
		docker rmi $(IMAGE_NAME); \
	else \
		echo "No such container: $(CONTAINER_NAME)"; \
	fi
	rm -f $(CHECKSUM_FILE)
	rm -f $(FILES_CHANGED_FLAG)

fclean: clean
	@if [ ! -z "$$(docker images -q $(IMAGE_NAME))" ]; then \
		docker rmi -f $(IMAGE_NAME); \
	else \
		echo "No such image: $(IMAGE_NAME)"; \
	fi

.PHONY: all docker-build docker-create docker-start transfer-and-build check-checksums update-checksums clean fclean