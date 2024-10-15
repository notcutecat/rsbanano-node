mod common;
mod ledger;
mod node;
mod utils;
mod wallets;

pub use common::*;
pub use ledger::*;
pub use node::*;
pub use utils::*;
pub use wallets::*;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum RpcCommand {
    AccountInfo(AccountInfoArgs),
    Keepalive(AddressWithPortArgs),
    Stop,
    KeyCreate,
    Receive(ReceiveArgs),
    Send(SendArgs),
    WalletAdd(WalletAddArgs),
    AccountCreate(AccountCreateArgs),
    AccountBalance(AccountBalanceArgs),
    AccountsCreate(AccountsCreateArgs),
    AccountRemove(WalletWithAccountArgs),
    AccountMove(AccountMoveArgs),
    AccountList(WalletRpcMessage),
    WalletCreate(WalletCreateArgs),
    WalletContains(WalletWithAccountArgs),
    WalletDestroy(WalletRpcMessage),
    WalletLock(WalletRpcMessage),
    WalletLocked(WalletRpcMessage),
    AccountBlockCount(AccountRpcMessage),
    AccountKey(AccountRpcMessage),
    AccountGet(KeyRpcMessage),
    AccountRepresentative(AccountRpcMessage),
    AccountWeight(AccountWeightArgs),
    AvailableSupply,
    BlockAccount(HashRpcMessage),
    BlockConfirm(HashRpcMessage),
    BlockCount,
    Uptime,
    FrontierCount,
    ValidateAccountNumber(AccountRpcMessage),
    NanoToRaw(AmountRpcMessage),
    RawToNano(AmountRpcMessage),
    WalletAddWatch(WalletAddWatchArgs),
    WalletRepresentative(WalletRpcMessage),
    WorkSet(WorkSetArgs),
    WorkGet(WalletWithAccountArgs),
    WalletWorkGet(WalletRpcMessage),
    AccountsFrontiers(AccountsRpcMessage),
    WalletFrontiers(WalletRpcMessage),
    Frontiers(FrontiersArgs),
    WalletInfo(WalletRpcMessage),
    WalletExport(WalletRpcMessage),
    PasswordChange(WalletWithPasswordArgs),
    PasswordEnter(WalletWithPasswordArgs),
    PasswordValid(WalletRpcMessage),
    DeterministicKey(DeterministicKeyArgs),
    KeyExpand(KeyExpandArgs),
    Peers(PeersArgs),
    PopulateBacklog,
    Representatives(RepresentativesArgs),
    AccountsRepresentatives(AccountsRpcMessage),
    StatsClear,
    UncheckedClear,
    Unopened(UnopenedArgs),
    NodeId,
    SearchReceivableAll,
    ReceiveMinimum,
    WalletChangeSeed(WalletChangeSeedArgs),
    Delegators(DelegatorsArgs),
    DelegatorsCount(AccountRpcMessage),
    BlockHash(BlockHashArgs),
    AccountsBalances(AccountsBalancesArgs),
    BlockInfo(HashRpcMessage),
    Blocks(HashesArgs),
    BlocksInfo(HashesArgs),
    Chain(ChainArgs),
    Successors(ChainArgs),
    ConfirmationActive(ConfirmationActiveArgs),
    ConfirmationQuorum(ConfirmationQuorumArgs),
    WorkValidate(WorkValidateArgs),
    AccountHistory(AccountHistoryArgs),
    Sign(SignArgs),
    Process(ProcessArgs),
    WorkCancel(HashRpcMessage),
    Bootstrap(BootstrapArgs),
    BootstrapAny(BootstrapAnyArgs),
    BoostrapLazy(BootsrapLazyArgs),
    WalletReceivable(WalletReceivableArgs),
    WalletRepresentativeSet(WalletRepresentativeSetArgs),
    SearchReceivable(WalletRpcMessage),
    WalletRepublish(WalletWithCountArgs),
    WalletBalances(WalletBalancesArgs),
    WalletHistory(WalletHistoryArgs),
    WalletLedger(WalletLedgerArgs),
    AccountsReceivable(AccountsReceivableArgs),
    Receivable(ReceivableArgs),
    ReceivableExists(ReceivableExistsArgs),
    RepresentativesOnline(RepresentativesOnlineArgs),
    Unchecked(CountRpcMessage),
    UncheckedGet(HashRpcMessage),
    UncheckedKeys(UncheckedKeysArgs),
    ConfirmationInfo(ConfirmationInfoArgs),
    Ledger(LedgerArgs),
    WorkGenerate(WorkGenerateArgs),
    Republish(RepublishArgs),
    BlockCreate(BlockCreateArgs),   
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RpcDto {
    AccountBalance(AccountBalanceDto),
    Account(AccountRpcMessage),
    Accounts(AccountsRpcMessage),
    Removed(RemovedDto),
    Moved(MovedDto),
    WalletCreate(WalletRpcMessage),
    KeyPair(KeyPairDto),
    Exists(ExistsDto),
    Error(ErrorDto),
    Destroyed(DestroyedDto),
    Locked(LockedDto),
    Lock(LockedDto),
    Stop(SuccessDto),
    AccountBlockCount(AccountBlockCountDto),
    AccountKey(KeyRpcMessage),
    AccountGet(AccountRpcMessage),
    AccountRepresentative(AccountRepresentativeDto),
    AccountWeight(WeightDto),
    AvailableSupply(AvailableSupplyDto),
    BlockConfirm(StartedDto),
    BlockCount(BlockCountDto),
    BlockAccount(AccountRpcMessage),
    Uptime(UptimeDto),
    Keepalive(StartedDto),
    FrontierCount(CountRpcMessage),
    ValidateAccountNumber(SuccessDto),
    NanoToRaw(AmountRpcMessage),
    RawToNano(AmountRpcMessage),
    WalletAddWatch(SuccessDto),
    WalletRepresentative(WalletRepresentativeDto),
    WorkSet(SuccessDto),
    WorkGet(WorkDto),
    WalletWorkGet(AccountsWithWorkDto),
    AccountsFrontiers(FrontiersDto),
    WalletFrontiers(FrontiersDto),
    Frontiers(FrontiersDto),
    WalletInfo(WalletInfoDto),
    WalletExport(JsonDto),
    PasswordChange(SuccessDto),
    PasswordEnter(ValidDto),
    PasswordValid(ValidDto),
    DeterministicKey(KeyPairDto),
    KeyExpand(KeyPairDto),
    Peers(PeersDto),
    PopulateBacklog(SuccessDto),
    Representatives(RepresentativesDto),
    AccountsRepresentatives(AccountsRepresentativesDto),
    StatsClear(SuccessDto),
    UncheckedClear(SuccessDto),
    Unopened(UnopenedDto),
    NodeId(NodeIdDto),
    Send(BlockDto),
    SearchReceivableAll(SuccessDto),
    ReceiveMinimum(AmountRpcMessage),
    WalletChangeSeed(WalletChangeSeedDto),
    Delegators(DelegatorsDto),
    DelegatorsCount(CountRpcMessage),
    BlockHash(HashRpcMessage),
    AccountsBalances(AccountsBalancesDto),
    BlockInfo(BlockInfoDto),
    Blocks(BlocksDto),
    BlocksInfo(BlocksInfoDto),
    Chain(BlockHashesDto),
    ConfirmationActive(ConfirmationActiveDto),
    ConfirmationQuorum(ConfirmationQuorumDto),
    WorkValidate(WorkValidateDto),
    AccountInfo(AccountInfoDto),
    AccountHistory(AccountHistoryDto),
    Sign(SignDto),
    Process(HashRpcMessage),
    WalletBalances(AccountsBalancesDto),
    WorkCancel(SuccessDto),
    Bootstrap(SuccessDto),
    BootstrapAny(SuccessDto),
    BootstrapLazy(BootstrapLazyDto),
    WalletReceivable(ReceivableDto),
    WalletRepresentativeSet(SetDto),
    SearchReceivable(ExistsDto),
    WalletRepublish(BlockHashesDto),
    WalletHistory(WalletHistoryDto),
    WalletLedger(WalletLedgerDto),
    AccountsReceivable(ReceivableDto),
    Receivable(ReceivableDto),
    ReceivableExists(ExistsDto),
    RepresentativesOnline(RepresentativesOnlineDto),
    Unchecked(UncheckedDto),
    UncheckedGet(UncheckedGetDto),
    UncheckedKeys(UncheckedKeysDto),
    ConfirmationInfo(ConfirmationInfoDto),
    Ledger(LedgerDto),
    WorkGenerate(WorkGenerateDto),
    Republish(BlockHashesDto),
    BlockCreate(BlockCreateDto)
}

