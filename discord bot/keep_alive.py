from flask import Flask, request
from threading import Thread
from database import *
app = Flask('')

@app.route("/")
def main():
  return "Hello"

def run():
  app.run(host="0.0.0.0", port=8080)

@app.route("/auth")
def linkblocks_auth():
    insert_auth_data(request.args.get("discord_id"), request.args.get("api_key"), request.args.get("user_id"))
    return "You are now authenticated! You can close this tab."
  
def keep_alive():
  server = Thread(target=run)
  server.start()