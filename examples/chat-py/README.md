## Chat example in Python

### Steps
- Create a .env file with the openai api key. Look into .env.sample for reference.
- Run the below commands to execute the chat app for the first time.

In the server root, run:
```bash
docker-compose build
docker-compose up
```
Then in this folder:
```bash
python3.11 -m venv venv
. ./venv/bin/activate
pip install -r requirements.txt
python main.py
```
