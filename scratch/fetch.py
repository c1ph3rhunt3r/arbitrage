import urllib.request, json, base64, subprocess

subprocess.run(['pip', 'install', 'base58'], stdout=subprocess.DEVNULL)
import base58

RPC='https://mainnet.helius-rpc.com/?api-key=3f1e002b-6b46-4a6d-bc0f-85503ded1e80'
KLEND='KLend2g3cP87fffoy8q1mQqGKjrxjC8boSyAYavgmjD'
MARKET='7u3HeHxYDLhnCoErrpiSYLS761M12zZDBpZ1mD2F7GfD'
WSOL='So11111111111111111111111111111111111111112'
USDC='EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v'

req = urllib.request.Request(RPC, data=json.dumps({
    'jsonrpc': '2.0', 'id': 1, 'method': 'getProgramAccounts',
    'params': [KLEND, {'encoding': 'base64', 'filters': [{'dataSize': 8624}]}]
}).encode(), headers={'content-type': 'application/json'})

import ssl
ctx = ssl.create_default_context()
ctx.check_hostname = False
ctx.verify_mode = ssl.CERT_NONE

res = json.loads(urllib.request.urlopen(req, context=ctx).read())

for acc in res['result']:
    data = base64.b64decode(acc['account']['data'][0])
    lending_market = base58.b58encode(data[24:56]).decode()
    if lending_market != MARKET: continue
    
    mint = base58.b58encode(data[72:104]).decode()
    if mint not in [WSOL, USDC]: continue
    
    supply = base58.b58encode(data[104:136]).decode()
    fee = base58.b58encode(data[136:168]).decode()
    
    print(f"Mint: {mint}")
    print(f"Reserve: {acc['pubkey']}")
    print(f"Supply: {supply}")
    print(f"FeeVault: {fee}\n")
