use fuels::prelude::abigen;
use fuels::programs::responses::CallResponse;

abigen!(Contract(
    name = "Proxy",
    abi = "contracts/proxy-contract/out/debug/proxy-contract-abi.json"
));

pub mod proxy_abi {
    use super::*;
    use fuels::prelude::{Account, ContractId, Error, TxPolicies};

    pub async fn set_proxy_target<T: Account + Clone>(
        proxy: &Proxy<T>,
        new_target: ContractId,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        proxy
            .methods()
            .set_proxy_target(new_target)
            .with_tx_policies(tx_params)
            .call()
            .await
    }

    pub async fn get_proxy_target<T: Account + Clone>(
        proxy: &Proxy<T>,
    ) -> Result<CallResponse<Option<ContractId>>, Error> {
        proxy.methods().proxy_target().call().await
    }

    pub async fn set_proxy_owner<T: Account + Clone>(
        proxy: &Proxy<T>,
        new_proxy_owner: State,
    ) -> Result<CallResponse<()>, Error> {
        let tx_params = TxPolicies::default().with_tip(1);

        proxy
            .methods()
            .set_proxy_owner(new_proxy_owner)
            .with_tx_policies(tx_params)
            .call()
            .await
    }

    pub async fn get_proxy_owner<T: Account + Clone>(
        proxy: &Proxy<T>,
    ) -> Result<CallResponse<State>, Error> {
        proxy.methods().proxy_owner().call().await
    }
}
