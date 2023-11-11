mod constant;

use chrono::{Duration, Utc};
use ethers::contract::abigen;
use ethers::middleware::SignerMiddleware;
use ethers::prelude::k256;
use ethers::signers::Wallet;
use ethers::types::{Address, U256};
use ethers::{
    providers::{Http, Provider},
    signers::LocalWallet,
};
use std::sync::Arc;

type Client = SignerMiddleware<Provider<Http>, Wallet<k256::ecdsa::SigningKey>>;

abigen!(DepositToken,
    "./src/abis/DepositToken.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

abigen!(
    CampaignFactory,
    "./src/abis/CampaignFactory.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

abigen!(
    Campaign,
    "./src/abis/Campaign.json",
    event_derives(serde::Deserialize, serde::Serialize)
);

struct ContractInteraction {
    client: Client,
}

impl ContractInteraction {
    fn new(client: Client) -> Self {
        Self { client }
    }

    async fn approve(&self, spender: &Address, amount: U256) -> Result<(), Box<dyn std::error::Error>> {
        let deposit_token = DepositToken::new(spender.clone(), Arc::new(self.client.clone()));

        let tx = deposit_token
            .approve(spender.clone(), amount)
            .send()
            .await?
            .await?;

        println!("Transaction Receipt: {}", serde_json::to_string(&tx)?);

        Ok(())
    }

    async fn create_campaign(
        &self,
        contract_addr: &Address,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let campaign_factory = CampaignFactory::new(contract_addr.clone(), Arc::new(self.client.clone()));

        let accepted_tokens = campaign_factory.accepted_token_addresses().call().await?;

        println!("Campaign factory accepted tokens are {:?}", accepted_tokens);
        let wallet: Address = self.client.address();
        let start_time = Utc::now() + Duration::seconds(10);
        let end_time = start_time + Duration::days(365);

        let tx = campaign_factory
            .create_campaign(campaign_factory::CreateCampaignParams {
                start_time: start_time.timestamp() as u64,
                end_time: end_time.timestamp() as u64,
                cliff_duration: 0,
                beneficiary: wallet,
                target_amount: 20000000000000000000u128.into(),
                asset: "0x7b4e9b59dc4280de59ec64a90ba666a887967279".parse()?,
                metadata: "0x".parse()?,
                // segments should be the param from CreateCampaignParams and it's length should > 0
                // percentage_bps should be 10000 with type U256
                segments: vec![
                    campaign_factory::Segment {
                        percentage_bps: 5000u128.into(),
                        milestone: (start_time + Duration::days(30)).timestamp() as u64,
                    },
                    campaign_factory::Segment {
                        percentage_bps: 5000u128.into(),
                        milestone: (start_time + Duration::days(60)).timestamp() as u64,
                    }],
            })
            .send()
            .await?
            .await?;

        println!("Transaction Receipt: {}", serde_json::to_string(&tx)?);

        Ok(())
    }

    async fn donate(&self, contract_addr: &Address, amount: U256) -> Result<(), Box<dyn std::error::Error>> {
        let campaign = Campaign::new(contract_addr.clone(), Arc::new(self.client.clone()));

        let tx = campaign.donate(amount).send().await?.await?;

        println!("Transaction Receipt: {}", serde_json::to_string(&tx)?);

        Ok(())
    }
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = Provider::<Http>::try_from(constant::BNB_TESTNET_RPC_URL)?;

    let signer: LocalWallet = constant::PRIVATE_KEY.parse::<LocalWallet>()?;

    let client = SignerMiddleware::new_with_provider_chain(provider.clone(), signer.clone()).await.unwrap();

    let contract_interaction = ContractInteraction::new(client.clone());

    // create a campaign
    // let factory_addr = FACTORY_ADDRESS.parse()?;
    // contract_interaction.create_campaign(&client, &factory_addr).await?;

    let campaign_addr = "0xfa4be5cbb918e92939171919d6dbbf5349813598".parse()?;

    let amount = 10000000000000000u128.into();

    // approve the campaign to spend 10000000000000000u128 so that the transferFrom will not fail
    contract_interaction.approve(&campaign_addr, amount).await?;

    // donate 10000000000000000u128 to the campaign
    contract_interaction.donate(&campaign_addr, amount).await?;

    Ok(())
}
