<div align="center">
  <a href="https://www.normalfinance.io/">
    <img src="https://cdn.prod.website-files.com/6595b2282ea917577755d3a5/6595bb9290625dfff5df3f7e_Logo%20-%20Color.svg" alt="Normal logo" width="340"/>
  </a>
</div>

<div>
  <a href="https://discord.gg/hayb9pafjZ"><img src="https://img.shields.io/discord/928701482319101952"/></a>
  <a  href="https://github.com/normalfinance/normal-index-v1/releases"><img src="https://img.shields.io/github/release-pre/normalfinance/normal-index-v1.svg"/></a>
  <a  href="https://github.com/normalfinance/normal-index-v1/pulse"><img src="https://img.shields.io/github/contributors/normalfinance/normal-index-v1.svg"/></a>
  <a href="https://opensource.org/license/apache-2-0"><img src="https://img.shields.io/github/license/normalfinance/normal-index-v1"/></a>
  <a href="https://github.com/normalfinance/normal-index-v1/pulse"><img src="https://img.shields.io/github/last-commit/normalfinance/normal-index-v1.svg"/></a>
  <a href="https://github.com/normalfinance/normal-index-v1/pulls"><img src="https://img.shields.io/github/issues-pr/normalfinance/normal-index-v1.svg"/></a>
 
  <a href="https://github.com/normalfinance/normal-index-v1/issues"><img src="https://img.shields.io/github/issues/normalfinance/normal-index-v1.svg"/></a>
  <a href="https://github.com/normalfinance/normal-index-v1/issues"><img src="https://img.shields.io/github/issues-closed/normalfinance/normal-index-v1.svg"/></a>
</div>

# ✨ Normal Index v1 🦄

Customizable on-chain index funds supporting any cryptocurrencies, commodities, equities, forex, and more.

## Features

- Diversify your choice of hundreds of global assets and asset classes with ease
- Create public index funds to invest with friends, family, or your community
- Create private index funds to invest personally or deploy unique strategies for clients
- Earn passive income when someone invests in your public index

## Todo

- [ ] Component obfuscation for private index funds (for proprietary strategies)
- [ ] Normal DAO management of public indexes (possibly using Soroban Governor)
- [ ] ...

## Smart Contracts

- **index_fund** - A DeFi primitive for diverisified portfolios using a basket of assets
- **index_fund_factory** - Factory for creating and interacting with Index Fund contracts
- **adapter_registry** - A registry tracking supported adapters for use with Index Funds

## Adapters

- **normal** - An adapter for buying and selling long and short Normal tokens via the Normal Treasury
- **aquarius** - An adapter for swapping tokens using Aquarius AMM pools
- **soroswap** - An adapter for swapping tokens using the Soroswap AMM Aggregator

## Modules

- **access_control** - Handles permissioned access to contracts using role-based authentication
- **adapter** - Defines the interface adapters must export to work with index funds
- **index_access_control** - Handles permissioned access to Index Fund contracts using role-based authentication
- **token_share** - Handles index token utilities
- **types** - Handles contract types
- **upgrade** - Handles contract upgrades
- **utils** - Handles shared types, utils, constants, errors, macros, and more

## Built With

- [Rust](https://www.rust-lang.org/)
- [Soroban](https://soroban.stellar.org/)
- [Rust Soroban SDK](https://github.com/stellar/rs-soroban-sdk)

## Getting Started

### Prerequisites

- [Task](https://taskfile.dev/) as task runner
- installed latest Rust version
- [soroban cli](https://github.com/stellar/soroban-tools)

### Development setup

#### Clone project

`git clone git@github.com:normalfinance/normal-index-v1.git`

#### Build contracts

`task build`

#### Run tests

`task test`

#### (Optionally) Deploy & invoke contracts via soroban-cli

check the Soroban documentation: https://soroban.stellar.org/docs/reference/rpc

## Authors

- [@jblewnormal](https://github.com/jblewnormal)
- [@jaymalve](ttps://github.com/jaymalve)

## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## Contact

- 📧 Email: [hello@normalfinance.io](mailto:hello@normalfinance.io)
- ✈️ Telegram: [@normalfinance](https://t.me/normalfinance)
- 🐣 Twitter: [@normalfi](https://twitter.com/normalfi)
- 🥷🏼 GitHub: [@normalfinance](https://github.com/normalfinance)
- 👾 Discord: [@Normal](https://discord.gg/xQMvceZjeS)
- 📚 Docs: [@normalfinance](https://normalfinance.gitbook.io/docs/)
- 🤓 Blog: [@normalfinance](https://blog.normalfinance.io/)

## License

[Apache-2.0](https://choosealicense.com/licenses/apache-2.0/)
