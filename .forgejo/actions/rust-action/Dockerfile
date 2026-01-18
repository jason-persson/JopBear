FROM node:22-trixie
RUN apt-get update && apt-get install -y rustup
RUN rustup default stable
COPY docker_entry.sh /docker_entry.sh
RUN chmod +x /docker_entry.sh
ENTRYPOINT ["/docker_entry.sh"]
