const express = require('express');
const {thorify} = require("thorify");
const Web3 = require("web3");
const app = express();
const web3 = thorify(new Web3(), "https://mainnet.veblocks.net");


app.get('/balance', async (req, res) => {
    const account = req.query.account;
    if (account.length !== 42) { // check for 20-byte address length
        return res.status(400).send("Wrong account length!")
    }

    const contractAbi = require("../abi.json");
    const contract = new web3.eth.Contract(contractAbi, "0x46209D5e5a49C1D403F4Ee3a0A88c3a27E29e58D");
    const accountBalance = await contract.methods.balanceOf(account).call();
    const formatted = web3.utils.fromWei(accountBalance, "ether");

    return res.status(200).send(formatted);
})

module.exports = {
    app
}