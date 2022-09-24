use yew::{function_component, html, Html, Properties};
use ethers::{prelude::*, utils::format_units};

#[derive(Properties, PartialEq)]
pub struct HistoryBlocksProps {
    pub blocks: Vec<Block<H256>>,
}

#[function_component(HistoryBlocks)]
pub fn prevs_table(props: &HistoryBlocksProps) -> Html {
    let list_of_blocks = &props.blocks;
    let sorted_txs: Vec<Block<H256>> = list_of_blocks
        .clone()
        .into_iter()
        .rev()
        .collect();
    let limited_txs = sorted_txs.iter().take(6); // change this to be selectable
    let prev_txs: Html = limited_txs
        .map(|tx| {
            html!{
                <tr key={tx.hash.as_ref().unwrap().to_string()}>
                    <td>
                        <a href={format!("https://etherscan.io/block/{}", tx.number.unwrap())} target={"blank"}>
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
        <table class="table">
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