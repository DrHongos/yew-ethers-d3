#![allow(dead_code)]
// get a network of txs where the account fetched is in the middle
// handle addresses (find ENS?)
// normalize values in parallel set.. otherwise plot are bs

use std::sync::Arc;
use std::collections::HashSet;
use ethers::prelude::*;
use ethers::prelude::account::ERC20TokenTransferEvent;
use ethers::utils::format_units;
use yew::prelude::*;
use web_sys::{HtmlInputElement};
use ethers::etherscan::{Client, account::TokenQueryOption};
use ethers::types::Chain;
use wasm_bindgen::prelude::*;
use serde::Serialize;

#[wasm_bindgen(module = "/src/jscripts/parallel_charts.js")]
extern "C" {
    #[wasm_bindgen(js_name = "ParallelCoordinates")]
    #[wasm_bindgen(catch)]
    pub fn ParallelCoordinates(data: JsValue, info: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name = "ParallelSet")]
    #[wasm_bindgen(catch)]
    pub fn ParallelSet(data: JsValue) -> Result<JsValue, JsValue>;

}

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
                self.history.truncate(0); // clean up
                //log::info!("Txs: {:?}", txs);
                self.history.extend(txs);
//                let (data, info_parsed) = self.get_parallel_coordinates_data();

                let data = self.get_parallel_set_data();

                //log::info!("Data len: {:?}", &data);
                match ParallelSet(data) {
  //              match ParallelCoordinates(data, info_parsed) {
                    Ok(_) => {                        
                        log::debug!("Plotting success!");
                        ()
                    },
                    Err(err) => {
                        log::error!("Plotting failed! {:?}", err);
                        ()
                    }
                };
            
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
            <div>
                <input 
                    ref={&self.search}
                    class={"input is-info"}
                    type={"text"}
                    placeholder={"Enter address or ENS"}
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
                    <p>{format!("{} ERC20 transactions", &self.history.len())}</p>
                    // {} 
                }
                if let Some(errors) = &self.error {
                    <p>{format!("Error: {}", errors)}</p>
                }
                <div id="parallel"></div>// class="is-fullheight"

            </div>
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct Sank {
    timestamp: String, 
    from: H160, 
    token_name: String,     
    to: H160,
    received: bool,
    value: String,
}

#[derive(Clone, Debug, Serialize)]
struct Tick {
    token_name: String, // only for the text
    contract_address: usize,
    timestamp: String,
    from: usize,//H160,
    to: usize,//H160,
    amount: String,
    received: bool, // otherwise transferred
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq, Hash)]
struct TokenData {
    contract_address: H160,
    token_name: String,
    //token_symbol: String,
}

// from & to should also share the 
//struct AddressData {
//    address: H160,
//    name: String, // ENS to fetch
//}

#[derive(Clone, Debug, Serialize)]
struct Information {
    tokens: Vec<TokenData>,
    from: Vec<String>,
    to: Vec<String>,
}
impl ERC20History {

    fn unique_tokens(&self) -> Vec<TokenData> {
        let r = &self.history;
        let tokens: Vec<TokenData> = r
            .into_iter()
            .map(|tx| {
                return 
                TokenData {
                    token_name: tx.token_name.clone(),
                    contract_address: tx.contract_address.clone(),
                    //token_symbol: tx.token_symbol.clone(),
                }
            })
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        tokens
    }
    fn unique_froms(&self) -> Vec<String> {
        let r = &self.history;
        let froms: Vec<String> = r
            .into_iter()
            .map(|tx| tx.from.to_string().clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        froms
    }
    fn unique_to(&self) -> Vec<String> {
        let r = &self.history;
        let tos: Vec<String> = r
            .into_iter()
            .map(|tx| tx.to.unwrap().to_string().clone())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        tos
    }
    fn get_info(&self) -> Information {
        Information {
            tokens: self.unique_tokens(),
            from: self.unique_froms(),
            to: self.unique_to(),
        }
    }
    fn get_parallel_set_data(&self) -> JsValue {
        let r = &self.history;
        let ticks = r
            .into_iter()
            .map(|x| return Sank {            
                token_name: x.token_name.clone(),
                timestamp: x.time_stamp.clone(),
                from: x.from.clone(),
                to: x.to.as_ref().unwrap().clone(),
                received: self.address.unwrap() == x.from,
                value: "1".to_string(),//format_units(x.value, x.token_decimal.parse::<i32>().unwrap()+4i32).unwrap(),
            })
            .collect::<Vec<Sank>>();
        let parsed = serde_wasm_bindgen::to_value(&ticks).unwrap();
        parsed
    }
    fn get_parallel_coordinates_data(&self) -> (JsValue, JsValue) {
        let r = &self.history;
        let info = self.get_info();
        //log::info!("info {:?}", info);
        let info_parsed = serde_wasm_bindgen::to_value(&info).unwrap();
        let ticks = r
            .into_iter()
            .map(|x| return Tick {
                token_name: x.token_name.clone(),
                timestamp: x.time_stamp.clone(),
                contract_address: info.tokens.iter().position(|r| r.contract_address == x.contract_address).unwrap(),//x.contract_address.clone(),
                from: info.from.iter().position(|r| *r == x.from.to_string()).unwrap(),//x.from.clone(),
                to: info.to.iter().position(|r| *r == x.to.unwrap().to_string()).unwrap(),//x.to.as_ref().unwrap().clone(),
                amount: format_units(x.value, x.token_decimal.parse::<i32>().unwrap()+4i32).unwrap(),
                received: self.address.unwrap() == x.from,
            })
            .collect::<Vec<Tick>>();
        //log::info!("Collected {} items", &ticks.len());
        let parsed = serde_wasm_bindgen::to_value(&ticks).unwrap();
        (parsed, info_parsed)
    }
}
