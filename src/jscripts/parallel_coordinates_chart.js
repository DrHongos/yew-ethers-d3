export function ParallelCoordinates(data, info) {
    const margin = ({top: 30, right: 30, bottom: 80, left: 50});
    const keys = ["timestamp", "from", "contract_address", "to"];//, "amount"
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

    //HELPERS
    const date_parsed = d => {
        let exits = new Date(d * 1000);
        let formatted = [
            exits.getDate().toString().padStart(2, '0'),
            (exits.getMonth() + 1).toString().padStart(2, '0'),
            exits.getFullYear(),
          ].join('/');        
        return formatted
    };
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
