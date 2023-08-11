# tglawbot: A Taiwanese (ROC) law searching bot for Telegram

## Usage 

1. Get bot token from botfather
```
echo "export TELOXIDE_TOKEN=<your_token>" > .env
```

2. Compile and start the program
```
. .env && cargo run
```

or 

```
docker build -t tglaw .
docker run -d tglaw
```
