# Local CortexDB memory

OpenHuman can set up and connect local CortexDB memory without asking users to run a server command, create a memory credential, or enter a network address.

## In the app

Open **Connections → Memory → Engine**. On a local session with no working memory engine, select **Start local memory**. OpenHuman calls `memory_local_start`, which:

1. Checks Docker Desktop or Docker Engine is installed and running.
2. Reuses an existing **OpenHuman-owned** CortexDB container and its stored data, or starts the pinned CortexDB Docker image (`cortexdb/cortexdb:v0.10.1`) on the first request.
3. Binds **only** `127.0.0.1:3141`, provisions a bearer credential and stores it in OpenHuman's credential store.
4. Waits for the server health endpoint, then selects the local engine in OpenHuman's existing memory configuration.

The backend runs in Docker, not inside the OpenHuman desktop executable. CortexDB currently provides Linux servers and multi-architecture Docker images rather than native macOS/Windows binaries. Docker is therefore an **optional prerequisite** for this local path. No download or background memory process starts merely because the user launches OpenHuman. Hosted TinyHumans memory and user-managed remote CortexDB remain alternatives.

## Data, resources, and accounts

The persistent Docker volume is `openhuman-cortexdb-data` and the container is named `openhuman-cortexdb`. Neither is deleted when the app exits or when you change memory providers. The container uses Docker's `unless-stopped` restart policy, so it can continue running when OpenHuman is closed. Stop it in Docker if you do not need it running; do not remove the named volume unless you intend to erase its data.

OpenHuman uses a separate `memory-cortexdb-local` credential identity for the automatically managed localhost endpoint, leaving an existing cloud CortexDB key untouched. A custom self-hosted endpoint is still available for advanced configurations. Only containers with the matching app ownership label are reused.

A default CortexDB instance offers basic memory features, but richer vector recall, fact extraction, and generated answers can require configured embedding/LLM providers. Provisioning a server is not equivalent to configuring every model capability. Keep the memory engine's reported health visible rather than assuming successful startup means every feature is ready.

## Troubleshooting

- **Docker missing or stopped:** OpenHuman displays the setup failure and a Docker installation/help link. Install or start Docker and retry.
- **Port already in use:** Free local TCP port 3141, then retry. OpenHuman will not reconfigure or remove someone else's container.
- **Health check timed out:** The initial image pull may take longer than subsequent launches; check Docker's container logs and retry.
- **Want to stop resource usage:** Stop `openhuman-cortexdb` in Docker. The app-owned volume stays intact, and **Start local memory** can restart the container when needed.

### Supported lifecycle

The current manager is **opt-in provisioning/reuse**, not native embedded database hosting or a full Docker dashboard. Before marking this feature production-ready, run desktop integration tests on Windows, macOS and Linux, including Docker unavailable, port collision, stopped container, retained volume, cloud-key preservation, and restart/recovery.
