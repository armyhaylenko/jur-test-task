const { app } = require("./server/server")
const port = process.env.SERVER_PORT || 3000;

app.listen(port, () => {
    console.log(`Express server listening on http://localhost:${port}`)
})