setInterval(() => {
    console.log(Math.random())
}, 1000);

const readline = require('readline');

const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
    terminal: false
});

rl.on('line', (line) => {
    console.log(`stdin: ${line}`);
});

rl.once('close', () => {
    // end of input
});
