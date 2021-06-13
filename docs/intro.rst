================
Introduction
================

RustPool is a `Monero <https://www.getmonero.org/>`_ mining pool server written in `Rust <https://www.rust-lang.org/>`_.
It offers an architecture optimized for the following deployment pattern in mind:

* Centralized pool state living in an SQL database (currently PostgreSQL).
* N stateless worker servers connecting to the database and Monero RPC client to process miner requests.
* One stateless backend server performing payments, chain state sync, and other periodic pool maintenance tasks.
* Containerized deployment of all servers using an orchestration system such as `Kubernetes <https://kubernetes.io/>`_.

This architecture offers a number of direct benefits for a pool operator. Having the pool's state live in an external
SQL database completely decouples the pool servers from UI concerns. This simplifies the server's code while
allowing the pool operator to expose any metrics they wish via normal SQL queries. Additionally, many cloud
providers offer multi-zone managed PostgreSQL clusters with backup/failover support, allowing an operator to
offload a large amount of the pain of state management to the cloud if they wish.
