//! Substrate Node Template CLI library.
#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;

fn main() -> sc_cli::Result<()> {
    let version = sc_cli::VersionInfo {
        name: "PKI Sample Node",
        commit: env!("VERGEN_SHA_SHORT"),
        version: env!("CARGO_PKG_VERSION"),
        executable_name: "nodle-pki-node",
        author: "Eliott Teissonniere <git.eliott@teissonniere.org>",
        description: "A node to demo the Nodle PKI runtime",
        support_url: "eliott@nodle.co",
        copyright_start_year: 2019,
    };

    command::run(version)
}
