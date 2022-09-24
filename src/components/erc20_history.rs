// al fondo!
// fetch erc20 history of an address
// displays something nice (stats, plot)
// timestamp of txs
// list of tokens
// list of address interacted with
// networks

use std::sync::Arc;
use std::collections::HashSet;
use ethers::prelude::*;
use ethers::prelude::account::ERC20TokenTransferEvent;
use yew::prelude::*;
use web_sys::{HtmlInputElement};
use ethers::etherscan::{Client, account::TokenQueryOption};
use ethers::types::Chain;
//use polars::prelude::*;

const API_MAINNET_KEY: &str = dotenv!("WSS_KEY_MAINNET");
const ETHERSCAN_API_KEY: &str = dotenv!("ETHERSCAN_API_KEY");
pub enum ERC20HistoryMsg {
    SetAddress(H160),
    SetEnsName(String),
    FetchAddress,
    FetchERC20Transactions,
    SetResult(Vec<ERC20TokenTransferEvent>),
    SetWsClient(Provider<Ws>),
    SetError(String),
}

pub struct ERC20History {
    ws_client: Option<Arc<Provider<Ws>>>,
    etherscan_client: Option<Client>,
    search: NodeRef, // string to search
    address: Option<H160>,
    ens_name: Option<String>,
    history: Vec<ERC20TokenTransferEvent>,
    error: Option<String>,
}

impl Component for ERC20History {
    type Message = ERC20HistoryMsg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async {
            match Ws::connect(API_MAINNET_KEY).await {
                Ok(ws) => {
                    let provider = Provider::new(ws);
//                    let client_ver = provider.client_version().await.unwrap();
//                    log::info!("Provider: {}",client_ver);
                    ERC20HistoryMsg::SetWsClient(provider)
                },
                Err(err) => {
                    log::error!("Error in connection to provider: {:?}", err);
                    ERC20HistoryMsg::SetError(err.to_string())
                }
            } 
        });
        match Client::new(Chain::Mainnet, ETHERSCAN_API_KEY) {
            Ok(client) => {
                Self {
                    ws_client: None,
                    etherscan_client: Some(client),
                    search: NodeRef::default(),
                    address: None,
                    ens_name: None,
                    history: Vec::new(),
                    error: None,
                }
            },
            Err(err) => {
                Self {
                    ws_client: None,
                    etherscan_client: None,
                    search: NodeRef::default(),                    
                    address: None,
                    ens_name: None,
                    history: Vec::new(),
                    error: Some(err.to_string()),
                }
            }
        }        
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ERC20HistoryMsg::SetWsClient(client) => {
                self.ws_client = Some(Arc::new(client));
                false
            }
            ERC20HistoryMsg::SetAddress(inp) => {
                log::info!("Address found {}", &inp.to_string());
                self.address = Some(inp);
                self.error = None; // clean up
                true
            }
            ERC20HistoryMsg::SetEnsName(inp) => {
                log::info!("ENS found {}", &inp);
                self.ens_name = Some(inp);
                true
            }            
            ERC20HistoryMsg::FetchAddress => {
                let val = self.search.cast::<HtmlInputElement>()
                    .unwrap()
                    .value()
                    .to_lowercase()
                    .to_owned();
                if val.ends_with(".eth") {
                    let p = self.ws_client.as_ref().unwrap().clone();
                    let v = val.clone();
                    ctx.link().send_future(async move {
                        match p.resolve_name(&v).await {
                            Ok(address) => {
                                ERC20HistoryMsg::SetAddress(address)
                                // handle address(0) "not taken"
                            },
                            Err(_err) => {
                                ERC20HistoryMsg::SetError("Provider error".to_string())
                            }
                        }
                    })
                } else {
                    match val.parse::<H160>() {
                        Ok(address) => {
                            let p = Arc::clone(self.ws_client.as_ref().unwrap());
                            let a1 = address.clone();
                            ctx.link().send_future(async move {
                                match p.lookup_address(a1).await {
                                    Ok(name) => ERC20HistoryMsg::SetEnsName(name),
                                    Err(err) => ERC20HistoryMsg::SetError(err.to_string())
                                }
                            });
                            ctx.link().send_message(ERC20HistoryMsg::SetAddress(address))
                        },
                        Err(_) => {
                            self.address = None;
                            ctx.link().send_message(ERC20HistoryMsg::SetError("Address wrong!".to_string()));
                        }
                    }
                }
                true
            }
            ERC20HistoryMsg::FetchERC20Transactions => {
                self.error = None;
                log::info!("To fetch address {:?}", &self.address);                                
                let address_copy = self.address.as_ref().unwrap().clone();
                if let Some(client) = self.etherscan_client.clone() {
                    ctx.link().send_future(async move {
                        match client.get_erc20_token_transfer_events(
                            TokenQueryOption::ByAddress(address_copy),
                            None
                        ).await {
                            Ok(txs) => {
                                ERC20HistoryMsg::SetResult(txs)
                            },
                            Err(err) => {
                                ERC20HistoryMsg::SetError(err.to_string())
                            }
                        }
                    });
                }
                false
            }
            ERC20HistoryMsg::SetResult(txs) => {
                self.history.truncate(0);
                log::info!("Txs: {:?}", txs);
                self.history.extend(txs);
                true
            }
            ERC20HistoryMsg::SetError(err) => {
                self.error = Some(err);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <span>
                {"Enter an EOA address: "}
                <input 
                    ref={&self.search}
                    class={"input is-info"}
                    type={"text"}
                    placeholder={"EOA Address"}
                    onchange={ctx.link().callback(|_| ERC20HistoryMsg::FetchAddress)}
                />
                if !&self.address.is_none() {
                    <p>{format!("Address accepted {}", &self.address.unwrap())}</p>
                    <button onclick={
                        ctx.link().callback(|_| ERC20HistoryMsg::FetchERC20Transactions)
                    }>{"Fetch ERC20 transactions"}</button>
                } 
                if let Some(ens) = &self.ens_name {
                    <p>{format!("ENS: {}", ens)}</p>
                }
                if self.history.len() > 0 {
                    <p>{format!("ERC20 tx total: {}", &self.history.len())}</p>
                    {self.network()}
                }
                if let Some(errors) = &self.error {
                    <p>{format!("Error: {}", errors)}</p>
                }
            </span>
        }
    }
}

//#[derive(Clone, Debug)]
//struct TokenMini {
//    contract: H160,
//    name: String,
//    symbol: String,
//}
impl ERC20History {
    fn network(&self) -> Html {
        // based on history or ERC20 txs
        let users: Vec<H160> = vec![];
        let r = &self.history;
        let tokens: Vec<String> = r
            .into_iter()
            .map(|tx| tx.token_name.clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        // need to dedup
        log::info!("tokens: {:?}", &tokens);
        // 
        html! {
            <>
                <p>{format!("You have interacted with {} addresses and with {} different ERC20 tokens.", 
                    users.len(),
                    tokens.len()
                )}</p>
                <p>{format!("only considers OUT txs.")}</p>
            </>
        }
    }
}