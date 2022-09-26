//HELPERS
function date_parsed(d) {
    let exits = new Date(d * 1000);
    let formatted = [
        exits.getDate().toString().padStart(2, '0'),
        (exits.getMonth() + 1).toString().padStart(2, '0'),
        exits.getFullYear(),
        ].join('/');        
    return formatted
};
function shorten_address(a) {
    let shortened = [a.slice(0, 4),a.slice(-5)].join("..");
    return shortened
}
function preparation(data, keys) {
    let index = -1;
    const nodes = [];
    const nodeByKey = new Map;
    const indexByKey = new Map;
    const links = [];
    
    for (const k of keys) {
        for (const d of data) {
        const key = JSON.stringify([k, d[k]]);
        if (nodeByKey.has(key)) continue;
        const node = {name: d[k]};
        nodes.push(node);
        nodeByKey.set(key, node);
        indexByKey.set(key, ++index);
        }
    }
    
    for (let i = 1; i < keys.length; ++i) {
        const a = keys[i - 1];
        const b = keys[i];
        const prefix = keys.slice(0, i + 1);
        const linkByKey = new Map;
        for (const d of data) {
        const names = prefix.map(k => d[k]);
        const key = JSON.stringify(names);
        const value = d.value || 1;
        let link = linkByKey.get(key);
        if (link) { link.value += value; continue; }
        link = {
            source: indexByKey.get(JSON.stringify([a, d[a]])),
            target: indexByKey.get(JSON.stringify([b, d[b]])),
            names,
            value
        };
        links.push(link);
        linkByKey.set(key, link);
        }
    }
    
    return {nodes, links};
};

export function ParallelCoordinates(data, info) {
    const margin = ({top: 30, right: 30, bottom: 80, left: 50});
    const keys = ["timestamp", "from", "contract_address", "to", "received"];//, "amount"
    const width = 1000;
    const height = keys.length * 200;
    let line = d3.line()
        .defined(([, value]) => value != null)
        .x(([key, value]) => x.get(key)(value))
        .y(([key]) => y(key));
    const y = d3.scalePoint(keys, [margin.top, height - margin.bottom]);
    const x = new Map(
        Array.from(
            keys, key => 
                [
                    key, d3.scaleLinear(d3.extent(data, d => d[key]), [margin.left, width - margin.right])
                ]
        )
    );    
    const keyz = "contract_address";//?
    const z = d3.scaleSequential(x.get(keyz).domain(), t => d3.interpolateBrBG(1 - t));
    d3.select("#parallel").select("svg").remove();
    const svg = d3
        .select("#parallel")
        .append("svg")
        .attr("width", width)
        .attr("height", height)
        .attr("viewBox", [0, 0, width, height])
        .attr("style", "max-width: 100%; height: auto; height: intrinsic;");
        
//    Object.entries(test).forEach(keyValuePair => {console.log("  ",...keyValuePair)})    

    svg.append("g")
        .attr("fill", "none")
        .attr("stroke-width", 5)
        .attr("stroke-opacity", 0.65)
        .selectAll("path")
        .data(data.slice().sort((a, b) => d3.ascending(a[keyz], b[keyz])))
        .join("path")
            .attr("stroke", d => z(d[keyz]))
            .attr("d", d => line(d3.cross(keys, [d], (key, d) => [key, d[key]])))            
        .append("title")
        .text(d => d.token_name); //

    svg.append("g")
        .selectAll("g")
        .data(keys)
        .join("g")
            .attr("transform", d => `translate(0,${y(d)})`)
            .style("font", "12px sans serif")
            .each(function(d) { 
                //console.log(`Here! setting axis of ${d}`);
                if (d === "timestamp") {
                    d3.select(this)
                        .call(
                            d3.axisBottom(x.get(d))
                                .tickFormat(t => {return date_parsed(t)})
                            )
                        .selectAll("text")  
                        .style("text-anchor", "end")
                        .attr("transform", "rotate(-45)");
                } else if (d === "from" || d === "to") {
                    d3.select(this)
                        .call(
                            d3.axisBottom(x.get(d))
                                .tickFormat(t => {
                                    return info[d][t]
                                }
                            )
                        )
                        .selectAll("text")  
                        .style("text-anchor", "end")
                        .attr("transform", "rotate(-45)");
                } else if ( d === "contract_address") {
                    d3.select(this)
                        .call(
                            d3.axisBottom(x.get(d))
                                .tickFormat(t => {                                    
                                    return info["tokens"][t]["token_name"]
                                }
                            )
                        )
                        .selectAll("text")  
                        .style("text-anchor", "end")
                        .attr("transform", "rotate(-45)");
                } else {
                    d3.select(this)
                        .call(d3.axisBottom(x.get(d))); 
                }
            })
            .call(g => g.append("text")
            .attr("x", margin.left)
            .attr("y", -6)
            .attr("text-anchor", "start")
            .attr("fill", "none")
            .text(d => d))
            .call(g => g.selectAll("text")
            //.clone(true)
            .lower()
            .attr("fill", "black")
            .attr("stroke-width", 1)
            .attr("stroke-linejoin", "round")
            .attr("stroke", "white"));

    return svg.node();

}

export function ParallelSet(data) {
    // https://observablehq.com/@d3/parallel-sets
    // experiment with two of these: RECEIVED:BALANCE:TRANSFERRED
    const margin = ({top: 30, right: 30, bottom: 80, left: 50});
    const keys = ["received", "timestamp", "from", "token_name", "to"];//amount and received not handled
    const width = 1000;
    const height = keys.length * 200;
    const color = d3.scaleOrdinal([true], ["#da4f81"]).unknown("green")
    const graph = preparation(data, keys);
    //console.log(`graph ${graph}`);
    //Object.entries(data).forEach(keyValuePair => {console.log("  ",...keyValuePair)})    
    const sankey = d3.sankey()
        .nodeSort(null)
        .linkSort(null)
        .nodeWidth(4)
        .nodePadding(20)
        .extent([[0, 5], [width, height - 5]])

    const svg = d3
        .select("#parallel")
        .append("svg")
        .attr("width", width)
        .attr("height", height)
        .attr("viewBox", [0, 0, width, height])
        .attr("style", "max-width: 100%; height: auto; height: intrinsic;");

    const {nodes, links} = sankey({
        nodes: graph.nodes.map(d => Object.assign({}, d)),
        links: graph.links.map(d => Object.assign({}, d))
    });

    svg.append("g")
        .selectAll("rect")
        .data(nodes)
        .join("rect")
            .attr("x", d => d.x0)
            .attr("y", d => d.y0)
            .attr("height", d => d.y1 - d.y0)
            .attr("width", d => d.x1 - d.x0)
        .append("title")
            .text(d => `${d.name}\n${d.value.toLocaleString()}`);

    svg.append("g")
        .attr("fill", "none")
        .selectAll("g")
        .data(links)
        .join("path")
            .attr("d", d3.sankeyLinkHorizontal())
            .attr("stroke", d => color(d.names[0]))
            .attr("stroke-width", d => d.width)
            .style("mix-blend-mode", "multiply")
        .append("title")
            .text(d => `${d.names.join(" → ")}\n${d.value.toLocaleString()}`);

    svg.append("g")
        .style("font", "20px sans-serif")
        .selectAll("text")
        .data(nodes)
        .join("text")
            .attr("x", d => d.x0 < width / 2 ? d.x1 + 6 : d.x0 - 6)
            .attr("y", d => (d.y1 + d.y0) / 2)
            .attr("dy", "0.35em")
            .attr("text-anchor", d => d.x0 < width / 2 ? "start" : "end")            
            .text(d => {
                d.name
            })
        .append("tspan")
            .attr("fill-opacity", 0.7)
            .text(d => {
                if (typeof d.name === "boolean") {
                    if (d.name) {return "Received"} else {return "Transferred"} 
                } else if (d.name.startsWith("0x")) {
                    return shorten_address(d.name)
                } else if (parseInt(d.name) > 0) {
                    return date_parsed(parseInt(d.name))
                } else if (typeof d.name === "string") {
                    return d.name
                } else {
                    return "test"
                }
            });

    return svg.node();
}
