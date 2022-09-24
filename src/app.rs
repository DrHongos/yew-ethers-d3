////////////////////////////////////////////////////////
//          TODO:
//      Input: EOA
    //      Allow fetching data of an account (metamask connected?)
    //      Fetch balance
    //      Fetch & display ERC20/ERC721/ERC1155 transfers
//      Input: Contract address
    //      Read ABI's of contracts (display handler?)
    //      Display stats about contracts (as a verifier)
//      Refactor
////////////////////////////////////////////////////////
#![allow(non_snake_case)]
use yew::prelude::*;
use ethers::{prelude::*, utils::format_units};
use std::sync::Arc;
use gloo_timers::callback::Interval;
use wasm_bindgen::prelude::*;
use serde::Serialize;
use crate::{components::{
    lineal_chart::LinealChart,
    last_block::LastBlock,
    history_blocks::HistoryBlocks,
    erc20_history::ERC20History,
}};
const API_MAINNET_KEY: &str = dotenv!("WSS_KEY_MAINNET");
pub enum AppMsg {
    SetClient(Provider<Ws>),
    FetchLastBlock,
    SetLastBlock(Block<H256>),    
    SetError(String),
    StartInterval,
//    FetchMempool,
//    UpdateMempool(TxpoolContent),
    StopInterval,
}
pub struct App {
    client: Option<Arc<Provider<Ws>>>,
    last_block: Option<Block<H256>>,    
    interval: Option<Interval>,
//    mempool_interval: Option<Interval>,
//    mempool: Option<TxpoolContent>,
    error: Option<String>,
    list_to_display: Vec<Block<H256>>,
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();
    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async {
            match Ws::connect(API_MAINNET_KEY).await {
                Ok(ws) => {
                    let provider = Provider::new(ws);
//                    let client_ver = provider.client_version().await.unwrap();
//                    log::info!("Provider: {}",client_ver);
                    AppMsg::SetClient(provider)
                },
                Err(err) => {
                    log::error!("Error in connection to provider: {:?}", err);
                    AppMsg::SetError(err.to_string())
                }
            } 
        });
        Self {
            client: None,
            last_block: None,
            error: None,
            interval: None,            
//            mempool: None,
//            mempool_interval: None,
            list_to_display: Vec::new(),
            //streamer: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {            
            AppMsg::FetchLastBlock => {
                let client = Arc::clone(&self.client.as_ref().unwrap());
                ctx.link().send_future(async move {
                    match client.get_block(BlockNumber::Latest).await {
                        Ok(block) => {
                            let block = block.unwrap();
                            AppMsg::SetLastBlock(block)
                        },
                        Err(err) => AppMsg::SetError(err.to_string())
                    }
                    });
                false
            }
//            AppMsg::FetchMempool => {
//                let client = Arc::clone(&self.client.as_ref().unwrap());
//                ctx.link().send_future(async move {
//                    match client
//                        .txpool_content()
//                        .await {
//                        Ok(mem) => {
//                            AppMsg::UpdateMempool(mem)
//                        },
//                        Err(err) => AppMsg::SetError(err.to_string())
//                    }
//                    });
//                false
//            }
//            AppMsg::UpdateMempool(TxpoolContent) => {
//                self.mempool = Some(TxpoolContent);
//                true
//            }
            AppMsg::StartInterval => {
                let ctx1 = ctx.link().clone();
                let handle = {
                    // blocks are 12s apart. Having 6 to update faster the list and maybe catch something?
                    Interval::new(6_000, move || ctx1.send_message(AppMsg::FetchLastBlock))
                };                
                self.interval = Some(handle);
// txpool calls are not supported by Infura nor Alchemy 
//                let ctx2 = ctx.link().clone();
//                let memp = {
//                    Interval::new(1_000, move || ctx2.send_message(AppMsg::FetchMempool))
//                };
//                self.mempool_interval = Some(memp);
                log::info!("Started interval");
                true
            }
            AppMsg::StopInterval => {
                //handler.cancel(); // HOW TO DEREFERENCE? IS THIS THE GWEI?
                self.interval = None;                // this works!
                //self.mempool_interval = None;
                log::info!("Stopped interval");
                true
            }
            AppMsg::SetClient(provider) => {
                self.client = Some(Arc::new(provider));
                ctx.link().send_message(AppMsg::FetchLastBlock); // first fetch
//                ctx.link().send_message(AppMsg::StartInterval); // auto-start
                true
            }
            AppMsg::SetLastBlock(bn) => {                            
                // moves last_block into list and updates the plot                
                if let Some(pb) = self.last_block.clone() {
                    // check that last_block is not in list
                    if !self.list_to_display.contains(&pb) {
                        self.list_to_display.push(pb);
                    }
                }
                self.last_block = Some(bn);
                true
            }
            AppMsg::SetError(err) => {
                self.error = Some(err);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <main class="container">
                if let Some(error) = &self.error {
                    <p>{ format!("Error: {:?}", error) }</p>
                }
                // separate next in two cols
                <div class="columns">
                    <div class="column">
                        <ERC20History />
                    </div>
                    <div class="column">                
                        if !self.client.is_none() {
                            if self.interval.is_none() {
                                <button onclick={ctx.link().callback(|_| AppMsg::StartInterval)}>
                                    {"Start interval"}
                                </button>
                            } else {
                                <button onclick={ctx.link().callback(|_| AppMsg::StopInterval)}>
                                    <div class={"spinner"}></div>
                                    {"Stop interval"}                            
                                </button>                
                            }
                        }
                        if self.list_to_display.len() != 0 {
                                <LinealChart
                                    list_to_display = {self.lineal_plot_data()}
                                />                    
                        }
                        <table class="table">
                            <tr>
                                <th> {"Last block"}</th>
                                <th> {"Previously, on mainnet.."}</th>
                            </tr>
                            <tr>
                            <td>
                                if let Some(last_block) = &self.last_block {
                                    <LastBlock
                                        last_block={last_block.clone()}
                                    />
                                } else {
                                    <p>{"Fetching.."}</p>
                                }
                            </td>
                            <td>
                                <HistoryBlocks
                                    blocks = {self.list_to_display.clone()} 
                                />
                            </td>
                            </tr>
                        </table>
                        if !self.client.is_none() {
                            <p>{"Provider (WS) connected"}</p>
                        }
                    </div>
                </div>
            </main>
        }
    }
    
}

#[derive(Debug, Serialize)]
pub struct Tick {
    pub x: u64,
    pub y: String, // hardcoded to base_fee_per_gas attribute
}

impl App {
    pub fn lineal_plot_data(&self) -> JsValue {
        let display: Vec<Tick> = self
            .list_to_display
            .iter()
            .map(|b| {
                return Tick {
                    x: b.timestamp.as_u64(), // convert to datetime?
                    y: format_units(b.base_fee_per_gas.unwrap_or_default(), 9).unwrap(),
                }
            })
            .collect();
        let parsed = serde_wasm_bindgen::to_value(&display).unwrap();
        parsed
    }

}