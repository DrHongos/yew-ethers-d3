#![allow(non_snake_case)]
use yew::prelude::*;
use ethers::{prelude::*, utils::format_units};
use std::sync::Arc;
use gloo_timers::callback::Interval;
use wasm_bindgen::prelude::*;
use serde::Serialize;

const API_MAINNET_KEY: &str = dotenv!("WSS_KEY_MAINNET");

#[wasm_bindgen(module = "/src/jscripts/lineal_chart.js")]
extern "C" {
    #[wasm_bindgen(js_name = "LineChart")]
    #[wasm_bindgen(catch)]
    pub fn LineChart(data: JsValue) -> Result<JsValue, JsValue>;
}

pub enum AppMsg {
    SetClient(Provider<Ws>),
    FetchLastBlock,
    SetLastBlock(Block<H256>),    
    SetError(String),
    StartInterval,
    FetchMempool,
    UpdateMempool(TxpoolContent),
    StopInterval,
}
pub struct App {
    client: Option<Arc<Provider<Ws>>>,
    last_block: Option<Block<H256>>,    
    interval: Option<Interval>,
    mempool_interval: Option<Interval>,
    mempool: Option<TxpoolContent>,
    error: Option<String>,
    list_to_display: Vec<Block<H256>>,
}
#[derive(Debug, Serialize)]
struct Tick {
    x: u64,
    y: String, // hardcoded to base_fee_per_gas attribute
}

impl Component for App {
    type Message = AppMsg;
    type Properties = ();
    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_future(async {
            match Ws::connect(API_MAINNET_KEY).await {
                Ok(ws) => {
                    let provider = Provider::new(ws);
                    let client_ver = provider.client_version().await.unwrap();
                    log::info!("Provider: {}",client_ver);
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
            mempool: None,
            mempool_interval: None,
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
            AppMsg::FetchMempool => {
                let client = Arc::clone(&self.client.as_ref().unwrap());
                ctx.link().send_future(async move {
                    match client
                        .txpool_content()
                        .await {
                        Ok(mem) => {
                            AppMsg::UpdateMempool(mem)
                        },
                        Err(err) => AppMsg::SetError(err.to_string())
                    }
                    });
                false
            }
            AppMsg::UpdateMempool(TxpoolContent) => {
                self.mempool = Some(TxpoolContent);
                true
            }
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
                self.mempool_interval = None;
                log::info!("Stopped interval");
                true
            }
            AppMsg::SetClient(provider) => {
                self.client = Some(Arc::new(provider));
                ctx.link().send_message(AppMsg::FetchLastBlock); // first fetch
                ctx.link().send_message(AppMsg::StartInterval); // auto-start
                true
            }
            AppMsg::SetLastBlock(bn) => {                            
                // moves last_block into list and updates the plot                
                if let Some(pb) = self.last_block.clone() {
                    // check that last_block is not in list
                    if !self.list_to_display.contains(&pb) {
                        self.list_to_display.push(pb);
                    }

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
                    let test = serde_wasm_bindgen::to_value(&display).unwrap();
                    match LineChart(test) {
                        Ok(_) => {                        
                            ()
                        },
                        Err(err) => {
                            log::error!("Plotting failed! {:?}", err);
                            AppMsg::SetError("Error on plotting".to_string());
                            ()
                        }
                    };
                }
                // sets new block (not plotted)
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
            <main>
                if let Some(error) = &self.error {
                    <p>{ format!("Error: {:?}", error) }</p>
                }

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
                <div id="chart" class="is-fullheight"></div>
                <table>
                    <tr>
                        <th> {"Last block"}</th>
                        <th> {"Previously, on mainnet.."}</th>
                    </tr>
                    <tr>
                    <td>
                        {self.table_block()}
                    </td>
                    <td>
                        {self.prevs_table()}
                    </td>
                    </tr>
                </table>
            </main>
        }
    }
    
}

impl App {
    pub fn table_block(&self) -> Html {
        if let Some(last_block) = &self.last_block {
            html! {
                <table>
                    <tr>
                        <th>{"key"}</th>
                        <th>{"value"}</th>
                    </tr>
                    <tr>
                        <td>{"number"}</td>
                        <td>
                            <a href={format!("https://etherscan.io/block/{}", last_block.number.unwrap())} target={"blank"} style={"color:white;"}>
                                {last_block.number.unwrap()}
                            </a>
                        </td>
                    </tr>
                    <tr>
                        <td>{"hash"}</td>
                        <td>{last_block.hash.unwrap()}</td>
                    </tr>
                    <tr>
                        <td>{"basefee"}</td>
                        <td>
                        {format!("{:.3} gwei", format_units(last_block.base_fee_per_gas.unwrap(), 9).unwrap())}
                        </td>
                    </tr>
                    <tr>
                        <td>{"gas used"}</td>
                        <td>{last_block.gas_used}</td>
                    </tr>
                    <tr>
                        <td>{"tx.len()"}</td>
                        <td>{last_block.transactions.len()}</td>
                    </tr>
                    <tr>
                        <td>{"size"}</td>
                        <td>{last_block.size.unwrap()} {" bytes"}</td>
                    </tr>
                </table>
            }
        } else {
            html! {}
        }
    }
    pub fn prevs_table(&self) -> Html {
        let sorted_txs: Vec<Block<H256>> = self.list_to_display
            .clone()
            .into_iter()
            .rev()
            .collect();
        let limited_txs = sorted_txs.iter().take(6);
        let prev_txs: Html = limited_txs
            .map(|tx| {
                html!{
                    <tr key={tx.hash.as_ref().unwrap().to_string()}>
                        <td>
                            <a href={format!("https://etherscan.io/block/{}", tx.number.unwrap())} target={"blank"} style={"color:white;"}>
                                {tx.number.unwrap()}
                            </a>
                        </td>
                        <td>{tx.hash.unwrap()}</td>
                        <td>{format!("{:.3} gwei", format_units(tx.base_fee_per_gas.unwrap(), 9).unwrap())}</td>
                        <td>{tx.gas_used}</td>
                        <td>{tx.transactions.len()}</td>
                        <td>{format!("{} bytes", tx.size.unwrap())}</td>
                    </tr>
                }
            }).collect();
        html! {
            <table>
                <tr>
                    <th>{"block number"}</th>
                    <th>{"hash"}</th>
                    <th>{"base fee"}</th>
                    <th>{"gas used"}</th>
                    <th>{"tx's qty"}</th>
                    <th>{"size"}</th>
                </tr>
                { prev_txs }
            </table>
        }
    }
}