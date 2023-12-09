IMAGE_NAME = alpinerustimage
CONTAINER_NAME = alpinerust
CHECKSUM_FILE = .checksums
FILES_CHANGED_FLAG = .files_changed

SRC_FILES = Cargo.toml i386-unknown-none.json linker.ld Makefile_docker
SRC_DIRS = src isofiles .cargo

all: docker-build docker-create docker-start check-checksums

docker-build:
	@if [ -z "$$(docker images -q $(IMAGE_NAME))" ]; then \
		echo "Building Docker image $(IMAGE_NAME)..."; \
		docker build -t $(IMAGE_NAME) .; \
	else \
		echo "Docker image $(IMAGE_NAME) already exists."; \
	fi

docker-create:
	@if [ -z "$$(docker ps -aq -f name=^$(CONTAINER_NAME)$$)" ]; then \
		echo "Creating Docker container $(CONTAINER_NAME)..."; \
		docker create --name $(CONTAINER_NAME) $(IMAGE_NAME); \
	else \
		echo "Docker container $(CONTAINER_NAME) already exists."; \
	fi

docker-start:
	@if [ -z "$$(docker ps -q -f name=^$(CONTAINER_NAME)$$ -f status=running)" ]; then \
		echo "Starting Docker container $(CONTAINER_NAME)..."; \
		docker start $(CONTAINER_NAME); \
	else \
		echo "Docker container $(CONTAINER_NAME) is already running."; \
	fi

transfer-and-build: check-checksums
	docker cp .cargo $(CONTAINER_NAME):/kfs
	docker cp isofiles $(CONTAINER_NAME):/kfs
	docker cp src $(CONTAINER_NAME):/kfs
	docker cp Cargo.toml $(CONTAINER_NAME):/kfs
	docker cp i386-unknown-none.json $(CONTAINER_NAME):/kfs
	docker cp linker.ld $(CONTAINER_NAME):/kfs
	docker cp Makefile_docker $(CONTAINER_NAME):/kfs/Makefile
	docker exec $(CONTAINER_NAME) make
	docker cp $(CONTAINER_NAME):/kfs/kfs.iso kfs.iso
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