use crate::RpcCommand;
use rsnano_core::Account;
use serde::{Deserialize, Serialize};

impl RpcCommand {
    pub fn account_balance(account_balance_args: AccountBalanceArgs) -> Self {
        Self::AccountBalance(account_balance_args)
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct AccountBalanceArgs {
    pub account: Account,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_unconfirmed_blocks: Option<bool>,
}

impl AccountBalanceArgs {
    pub fn builder(account: Account) -> AccountBalanceArgsBuilder {
        AccountBalanceArgsBuilder::new(account)
    }
}

impl From<Account> for AccountBalanceArgs {
    fn from(account: Account) -> Self {
        Self {
            account,
            include_unconfirmed_blocks: Some(false),
        }
    }
}

pub struct AccountBalanceArgsBuilder {
    args: AccountBalanceArgs,
}

impl AccountBalanceArgsBuilder {
    fn new(account: Account) -> Self {
        Self {
            args: AccountBalanceArgs {
                account,
                include_unconfirmed_blocks: None,
            },
        }
    }

    pub fn include_unconfirmed_blocks(mut self) -> Self {
        self.args.include_unconfirmed_blocks = Some(true);
        self
    }

    pub fn finish(self) -> AccountBalanceArgs {
        self.args
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::to_string_pretty;

    #[test]
    fn serialize_account_balance_command_include_unconfirmed_blocks() {
        let account_balance_args = AccountBalanceArgsBuilder::new(Account::zero())
            .include_unconfirmed_blocks()
            .finish();
        assert_eq!(
            to_string_pretty(&RpcCommand::account_balance(account_balance_args)).unwrap(),
            r#"{
  "action": "account_balance",
  "account": "nano_1111111111111111111111111111111111111111111111111111hifc8npp",
  "include_unconfirmed_blocks": true
}"#
        )
    }

    #[test]
    fn deserialize_account_balance_command_include_unconfirmed_blocks() {
        let json = r#"{
            "action": "account_balance",
            "account": "nano_111111111111111111111111111111111111111111111111115uwdgas549",
            "include_unconfirmed_blocks": true
        }"#;

        let deserialized: RpcCommand = serde_json::from_str(json).unwrap();

        if let RpcCommand::AccountBalance(args) = deserialized {
            assert_eq!(args.account, Account::from(123));
            assert_eq!(args.include_unconfirmed_blocks, Some(true));
        } else {
            panic!("Deserialized to wrong RpcCommand variant");
        }
    }

    #[test]
    fn serialize_account_balance_command_default() {
        let account_balance_args = AccountBalanceArgsBuilder::new(Account::zero()).finish();
        assert_eq!(
            to_string_pretty(&RpcCommand::account_balance(account_balance_args)).unwrap(),
            r#"{
  "action": "account_balance",
  "account": "nano_1111111111111111111111111111111111111111111111111111hifc8npp"
}"#
        )
    }

    #[test]
    fn deserialize_account_balance_command_default() {
        let account_balance_args = AccountBalanceArgsBuilder::new(Account::zero()).finish();
        let cmd = RpcCommand::account_balance(account_balance_args);
        let serialized = serde_json::to_string_pretty(&cmd).unwrap();
        let deserialized: RpcCommand = serde_json::from_str(&serialized).unwrap();
        assert_eq!(cmd, deserialized)
    }

    #[test]
    fn account_balance_args_from_account() {
        let account = Account::from(123);
        let args: AccountBalanceArgs = account.into();

        assert_eq!(args.account, account);
        assert_eq!(args.include_unconfirmed_blocks, Some(false));
    }
}
