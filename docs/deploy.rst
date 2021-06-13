================
Deployment
================

RustPool makes a few assumptions that must be adhered to when deploying. One is that the pool has two types of servers,
worker and backend, configured via the ``node_type`` configuration value. Worker servers are the servers that miners
connect to. They can connect to the central database and the Monero RPC daemon. The backend server performs payments
(if configured), chain state sync, and a variety of maintenance tasks periodically. It can connect to the database,
RPC daemon, and (if performing payments), the RPC wallet. Although there can be many workers in a pool, it is
assumed that there will always be only one backend server.
