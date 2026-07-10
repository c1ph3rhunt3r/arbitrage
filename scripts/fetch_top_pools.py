import urllib.request
import json
import time

def fetch_top_pools():
    print("Fetching top verified tokens from Jupiter...")
    try:
        # Get top tokens from Jupiter to seed our search
        req = urllib.request.Request("https://tokens.jup.ag/tokens?tags=verified", headers={'User-Agent': 'Mozilla/5.0'})
        with urllib.request.urlopen(req) as response:
            tokens = json.loads(response.read().decode())
    except Exception as e:
        print(f"Failed to fetch Jupiter tokens: {e}")
        return

    # Grab the top 30 tokens by daily volume (or just the first 30)
    top_tokens = [t["address"] for t in tokens[:30]]
    
    # Also add WSOL and USDC explicitly just in case
    wsol = "So11111111111111111111111111111111111111112"
    usdc = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
    if wsol not in top_tokens: top_tokens.append(wsol)
    if usdc not in top_tokens: top_tokens.append(usdc)

    all_pools = {
        "raydium": [],
        "meteora": [],
        "orca": [],
        "pump": []
    }

    seen_pairs = set()

    print("Scraping DexScreener for active pairs...")
    
    # For each token, get its top pairs
    for token in top_tokens:
        try:
            req = urllib.request.Request(f"https://api.dexscreener.com/latest/dex/tokens/{token}", headers={'User-Agent': 'Mozilla/5.0'})
            with urllib.request.urlopen(req) as response:
                data = json.loads(response.read().decode())
                
            if "pairs" not in data or data["pairs"] is None:
                continue
                
            for pair in data["pairs"]:
                pair_addr = pair["pairAddress"]
                if pair_addr in seen_pairs:
                    continue
                    
                seen_pairs.add(pair_addr)
                dex_id = pair.get("dexId", "")
                
                if dex_id == "raydium":
                    all_pools["raydium"].append(pair_addr)
                elif dex_id == "meteora":
                    all_pools["meteora"].append(pair_addr)
                elif dex_id == "orca":
                    all_pools["orca"].append(pair_addr)
                elif "pump" in dex_id.lower():
                    all_pools["pump"].append(pair_addr)
                    
            time.sleep(0.5) # rate limit
        except Exception as e:
            print(f"Error fetching token {token}: {e}")
            
        # Stop if we have a decent chunk
        if len(all_pools["raydium"]) > 150 and len(all_pools["meteora"]) > 50 and len(all_pools["orca"]) > 20:
            break

    # Cap at 200 each
    all_pools["raydium"] = all_pools["raydium"][:200]
    all_pools["meteora"] = all_pools["meteora"][:200]
    all_pools["orca"] = all_pools["orca"][:200]
    all_pools["pump"] = all_pools["pump"][:200]

    print(f"Found {len(all_pools['raydium'])} Raydium, {len(all_pools['meteora'])} Meteora, {len(all_pools['orca'])} Orca, {len(all_pools['pump'])} Pump pools.")
    
    with open("pools.json", "w") as f:
        json.dump(all_pools, f, indent=4)
    print("Saved to pools.json")

if __name__ == "__main__":
    fetch_top_pools()
