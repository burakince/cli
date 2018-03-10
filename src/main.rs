#[macro_use]
extern crate clap;
extern crate conv;
#[macro_use]
extern crate failure;
extern crate gpgme;
#[macro_use]
extern crate lazy_static;
extern crate sheesy_substitute as substitute;
extern crate sheesy_vault as vault;

mod cli;
mod parse;
mod dispatch;

use clap::ArgMatches;
use substitute::substitute;
use failure::Error;
use std::io::{stderr, stdout, Write};
use std::process;
use std::convert::Into;
use cli::CLI;
use vault::error::{first_cause_of_type, DecryptionError};
use vault::print_causes;

fn amend_error_info<T>(r: Result<T, Error>) -> Result<T, Error> {
    r.map_err(|e| {
        let ctx = match first_cause_of_type::<DecryptionError>(&e) {
            Some(_err) => Some(format!(
                "Export your public key using '{} vault recipient init', then \
                 ask one of the existing recipients to import your public key \
                 using '{} vault recipients add <your-userid>.'",
                CLI::name(),
                CLI::name()
            )),
            None => None,
        };
        (e, ctx)
    }).map_err(|(e, msg)| match msg {
        Some(msg) => e.context(msg).into(),
        None => e,
    })
}

fn ok_or_exit<T, E>(r: Result<T, E>) -> T
where
    E: Into<Error>,
{
    match r {
        Ok(r) => r,
        Err(e) => {
            write!(stderr(), "error: ").ok();
            print_causes(e, stderr());
            process::exit(1);
        }
    }
}

fn usage_and_exit(args: &ArgMatches) -> ! {
    println!("{}", args.usage());
    process::exit(1)
}

fn main() {
    let cli = CLI::new();
    let appc = cli.app.clone();
    let matches: ArgMatches = cli.app.get_matches();

    let res = match matches.subcommand() {
        ("completions", Some(args)) => parse::completions::generate(appc, args),
        ("substitute", Some(args)) => {
            let context = ok_or_exit(parse::substitute::context_from(args));
            substitute(context.data, &context.specs)
        }
        ("vault", Some(args)) => {
            let mut context = ok_or_exit(parse::vault::vault_context_from(args));
            context = match args.subcommand() {
                ("partitions", Some(args)) => match args.subcommand() {
                    ("add", Some(args)) => ok_or_exit(parse::vault::vault_partitions_add(context, args)),
                    ("remove", Some(args)) => ok_or_exit(parse::vault::vault_partitions_remove(context, args)),
                    _ => usage_and_exit(&matches),
                },
                ("recipients", Some(args)) => match args.subcommand() {
                    ("add", Some(args)) => ok_or_exit(parse::vault::vault_recipients_add(context, args)),
                    ("remove", Some(args)) => ok_or_exit(parse::vault::vault_recipients_remove(context, args)),
                    ("init", Some(args)) => ok_or_exit(parse::vault::vault_recipients_init(context, args)),
                    ("list", Some(args)) => ok_or_exit(parse::vault::vault_recipients_list(context, args)),
                    _ => ok_or_exit(parse::vault::vault_recipients_list(context, args)),
                },
                ("init", Some(args)) => ok_or_exit(parse::vault::vault_init_from(context, args)),
                ("add", Some(args)) => ok_or_exit(parse::vault::vault_resource_add(context, args)),
                ("remove", Some(args)) => ok_or_exit(parse::vault::vault_resource_remove(context, args)),
                ("show", Some(args)) => ok_or_exit(parse::vault::vault_resource_show(context, args)),
                ("edit", Some(args)) => ok_or_exit(parse::vault::vault_resource_edit(context, args)),
                ("list", Some(args)) => ok_or_exit(parse::vault::vault_resource_list(context, args)),
                _ => context,
            };
            let sout = stdout();
            let mut lock = sout.lock();
            amend_error_info(dispatch::vault::do_it(context, &mut lock))
        }
        _ => usage_and_exit(&matches),
    };

    ok_or_exit(res);
}
