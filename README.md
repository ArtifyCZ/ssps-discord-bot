# SSPŠ Discord bot

A Discord bot implemented in Rust that provides custom features for the
[official Discord server](https://discord.com/invite/ePGN2XrkJU) of
the [Smíchov Secondary Technical School (SSPŠ)](https://ssps.cz).

## Features

- Updating the server's information channel
- Verification of students using the school's Azure Active Directory
- Providing member information for server administrators

# Contributing

Contributions are welcome!

Please follow the set architecture and code style. The project uses `rustfmt` for formatting and `clippy` for linting.
All pull requests are required to pass the CI checks.

## Usage locally

To use the bot locally (for development purposes), you need to set up a `.env` file in the root directory of the project.
To find out what variables are required to be set, check the `.env.example` file, or the Ansible configuration.

## Usage remotely

To use the bot on a remote server (for production or testing purposes), you can use the provided
[Ansible playbook](ansible/deploy.yaml). You will need to define your own inventory file and variables.

# License

This project is licensed under the MIT license. For details see [LICENSE-MIT.txt](LICENSE-MIT.txt).
