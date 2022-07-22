Indexer that catches txs for specific contract(s)
=================================================

The most common use case for indexers is to react on a transaction sent to a specific contract or a list of contracts.

This project is trying to provide an example of the indexer described about. It's simple yet doing the necessary stuff. In this example we don't use any external storage (like database or files) to keep track for the transactions to keep the example as simple as possible.

We've tried to put the explanatory comments in the code to help developers to extend this example according to their needs.


> Please refer to [NEAR Indexer for Explorer](https://github.com/near/near-indexer-for-explorer) to find an inspiration for extending the indexer.


## How it works

Assuming we want to watch for transactions where a receiver account id is one of the provided in a list.
We pass the list of account ids (or contracts it is the same) via argument `--accounts`.
We want to catch all *successfull* transactions sent to one of the accounts from the list.
In the demo we'll just look for them and log them but it might and probably should be extended based on your needs.

---

## How to use

Before you proceed, make sure you have the following software installed:
* [rustup](https://rustup.rs/) or Rust version that is mentioned in `rust-toolchain` file in the root of [nearcore](https://github.com/nearprotocol/nearcore) project.

Clone this repository and open the project folder

```bash
$ git clone git@github.com:khorolets/indexer-tx-watcher-example.git
$ cd indexer-tx-watcher-example
```

### Init

To connect NEAR Indexer for Explorer to the specific chain you need to have necessary configs, you can generate it as follows:

* `localnet` (recommended to start with)
    ```bash
    $ cargo build --release
    $ ./target/release/indexer-tx-watcher-example --home-dir ~/.near/localnet init --chain-id localnet
    ```
* `testnet` (once you've adjusted and extended the example for your needs)
    ```bash
    $ cargo build --release
    $ ./target/release/indexer-tx-watcher-example --home-dir ~/.near/testnet init --chain-id testnet --download-config --download-genesis
    ```

The above code will download the official genesis config and generate necessary configs.

**NB!** According to changes in `nearcore` config generation we don't fill all the necessary fields in the config file.
While this issue is open https://github.com/nearprotocol/nearcore/issues/3156 you need to download config you want and replace the generated one manually.
 - [testnet config.json](https://s3-us-west-1.amazonaws.com/build.nearprotocol.com/nearcore-deploy/testnet/config.json)
 - [betanet config.json](https://s3-us-west-1.amazonaws.com/build.nearprotocol.com/nearcore-deploy/betanet/config.json)
 - [mainnet config.json](https://s3-us-west-1.amazonaws.com/build.nearprotocol.com/nearcore-deploy/mainnet/config.json)

Configs for the specified network are in the `--home-dir` provided folder. We need to ensure that NEAR Indexer for Explorer follows
all the necessary shards, so `"tracked_shards"` parameters in `~/.near/localnet/config.json` needs to be configured properly.
For example, with a single shared network, you just add the shards numbers you need to the list:

```
...
"tracked_shards": [0],
...
```

### Run

```bash
$ ./target/release/indexer-tx-watcher-example --home-dir ~/.near/localnet run --accounts mycoolcontract.near,myanothercoolcontract.near
```

Provide your contracts list after `--accounts` key separated with comma (`,`) **avoid spaces**

---

## Syncing

Whenever you run the indexer for any network except `localnet` you'll need to sync with the network. This is required because it's a natural behavior of `nearcore` node and the indexer is a wrapper for the regular `nearcore` node. In order to work and index the data your node must be synced with the network. This process can take a while, so we suggest to download a fresh backup of the `data` folder and put it in you `--home-dir` of your choice (by default it is `~/.near`)

Running your indexer node on top of a backup data will reduce the time of syncing process because your node will download only missing data and it will take reasonable time.

All the backups can be downloaded from the public S3 bucket which contains latest daily snapshots:

* [Recent 5-epoch Mainnet data folder](https://near-protocol-public.s3.ca-central-1.amazonaws.com/backups/mainnet/rpc/data.tar)
* [Recent 5-epoch Testnet data folder](https://near-protocol-public.s3.ca-central-1.amazonaws.com/backups/testnet/rpc/data.tar)


## Archival node

It's not necessary but in order to index everything in the network it is better to do it from the genesis. `nearcore` node is running in non-archival mode by default. That means that the node keeps data only for [5 last epochs](https://docs.near.org/concepts/basics/epoch). In order to index data from the genesis we need to turn the node in archival mode.

To do it we need to update `config.json` located in `--home-dir` or your choice (by default it is `~/.near`).

Find next keys in the config and update them as following:

```json
{
  ...
  "archive": true,
  "tracked_shards": [0],
  ...
}
```

The syncing process in archival mode can take a lot of time, so it's better to download a backup provided by NEAR and put it in your `data` folder. After that your node will need to sync only missing data and it should take reasonable time.

All the backups can be downloaded from the public S3 bucket which contains latest daily snapshots:

* [Archival Mainnet data folder](https://near-protocol-public.s3.ca-central-1.amazonaws.com/backups/mainnet/archive/data.tar)
* [Archival Testnet data folder](https://near-protocol-public.s3.ca-central-1.amazonaws.com/backups/testnet/archive/data.tar)

See [this article](https://docs.near.org/integrator/exchange-integration#running-an-archival-node) for reference
