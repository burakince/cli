use base::{Vault, VaultKind};
use failure::{err_msg, Error};
use std::io::Write;
use std::path::Path;
use sheesy_types::WriteMode;
use std::iter::once;

impl Vault {
    pub fn serialize(&self) -> Result<(), Error> {
        self.to_file(
            self.vault_path
                .as_ref()
                .map(|p| p.as_path())
                .ok_or_else(|| err_msg("Expected vault to know its configuration file"))?,
            WriteMode::AllowOverwrite,
        ).map_err(Into::into)
    }

    pub fn partition_index(selector: &str, partitions: &[Vault], leader_index: Option<usize>) -> Result<usize, Error> {
        let index: Result<usize, _> = selector.parse();
        Ok(match index {
            Ok(index) => {
                if let Some(leader_index) = leader_index {
                    if leader_index == index {
                        bail!(
                            "Refusing to remove the leading partition at index {}",
                            index
                        )
                    }
                };
                partitions
                    .iter()
                    .find(|v| v.index == index)
                    .map(|v| v.index)
                    .ok_or_else(|| format_err!("Could not find partition with index {}", index))?
            }
            Err(_) => {
                let selector_as_path = Path::new(selector);
                let mut matches = partitions.iter().filter_map(|v| {
                    if v.secrets.as_path() == selector_as_path {
                        Some(v.index)
                    } else {
                        v.name
                            .as_ref()
                            .and_then(|n| if n == selector { Some(v.index) } else { None })
                    }
                });
                match (matches.next(), matches.next()) {
                    (Some(index), None) => index,
                    (Some(_), Some(_)) => bail!(
                        "Multiple partitions matched the ambiguous selector '{}'",
                        selector
                    ),
                    _ => bail!("No partition matched the given selector '{}'", selector),
                }
            }
        })
    }

    pub fn remove_partition(&mut self, selector: &str, output: &mut Write) -> Result<(), Error> {
        let index = Vault::partition_index(selector, &self.partitions, Some(self.index))?;

        self.partitions.retain(|v| v.index != index);
        self.serialize()?;

        writeln!(output, "Removed partition matching selector '{}'", selector).ok();
        Ok(())
    }
    pub fn add_partition(&mut self, path: &Path, name: Option<&str>, output: &mut Write) -> Result<(), Error> {
        let secrets_dir = self.secrets.parent().ok_or_else(|| {
            format_err!(
                "Expected vault to have secrets directory ('{}') from which a parent directory can be obtained.",
                self.secrets.display()
            )
        })?;
        let partition_secrets_dir = secrets_dir.join(path);
        let recipients_file = partition_secrets_dir.join(self.recipients.file_name().ok_or_else(|| {
            format_err!(
                "Expected vault to have a recipients file ('{}') from which a filename can be obtained",
                self.recipients.display()
            )
        })?);
        let max_index = self.partitions
            .iter()
            .map(|v| v.index)
            .chain(once(self.index))
            .max()
            .expect("at least one item");
        self.partitions.push(Vault {
            name: name.map(ToOwned::to_owned)
                .or_else(|| path.file_name().map(|f| f.to_string_lossy().into_owned())),
            kind: VaultKind::Partition,
            index: max_index + 1,
            partitions: Vec::new(),
            resolved_at: self.resolved_at.clone(),
            vault_path: self.vault_path.clone(),
            secrets: partition_secrets_dir.clone(),
            gpg_keys: None,
            recipients: recipients_file,
        });

        self.serialize()?;
        match name {
            Some(name) => writeln!(
                output,
                "Added partition '{}' with resources at '{}'",
                name,
                partition_secrets_dir.display()
            ),
            None => writeln!(
                output,
                "Added unnamed partition with resources at '{}'",
                partition_secrets_dir.display()
            ),
        }.ok();
        Ok(())
    }
}
