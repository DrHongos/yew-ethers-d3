// using use_state inputs and address
// or ENS
// fetch the address (is EOA?) (has ENS?)
// if correct
// fetch erc20 history of an address
// displays something nice (stats, plot)
// else notice error

use ethers::prelude::account::ERC20TokenTransferEvent;
use yew::prelude::*;
use web_sys::{HtmlInputElement};
use ethers::etherscan::{Client, account::TokenQueryOption};
use ethers::types::Chain;

const ETHERSCAN_API_KEY: &str = dotenv!("ETHERSCAN_API_KEY");
pub enum ERC20HistoryMsg {
    SetAddress,
    FetchAddress,
    SetResult(Vec<ERC20TokenTransferEvent>),
    SetError(String),
}

pub struct ERC20History {
    client: Option<Client>,
    search: NodeRef, // string to search
    address: Option<String>,
    history: Vec<ERC20TokenTransferEvent>,
    error: Option<String>,
}

impl Component for ERC20History {
    type Message = ERC20HistoryMsg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        match Client::new(Chain::Mainnet, ETHERSCAN_API_KEY) {
            Ok(client) => {
                Self {
                    client: Some(client),
                    search: NodeRef::default(),
                    address: None,
                    history: Vec::new(),
                    error: None,
                }
            },
            Err(err) => {
                Self {
                    client: None,
                    search: NodeRef::default(),
                    address: None,
                    history: Vec::new(),
                    error: Some(err.to_string()),
                }
            }
        }        
    }
    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            ERC20HistoryMsg::SetAddress => {
                let val = &self.search.cast::<HtmlInputElement>()
                    .unwrap()
                    .value()
                    .to_lowercase();
                log::info!("Entered: {:?}", &val);
                // validation
                if val.starts_with("0x") && val.len() > 3 { // CAREFUL WITH ENS (later)
                    // call fetch
                    self.address = Some(String::from(val));
                    ctx.link().send_message(ERC20HistoryMsg::FetchAddress);
                } else {
                    ctx.link().send_message(ERC20HistoryMsg::SetError("Address wrong!".to_string()));
                }

                true
            }
            ERC20HistoryMsg::FetchAddress => {
                self.error = None;
                log::info!("To fetch address {:?}", &self.address);                                
                let address_copy = self.address.as_ref().unwrap().clone();
                if let Some(client) = self.client.clone() {
                    ctx.link().send_future(async move {
                        match client.get_erc20_token_transfer_events(
                            TokenQueryOption::ByAddress(address_copy.parse().unwrap()),
                            None
                        ).await {
                            Ok(txs) => {
                                log::info!("passed {:?}", &txs);
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
                log::info!("Txs: {:?}", txs);
                self.history = txs;
                false
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
                    onchange={ctx.link().callback(|_| ERC20HistoryMsg::SetAddress)}
                />
                if let Some(errors) = &self.error {
                    <p>{format!("Error: {}", errors)}</p>
                }
            </span>
        }
    }


}