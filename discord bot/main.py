import discord
from discord.ext import commands
from dotenv import load_dotenv
import random
import os
import keep_alive
import requests
load_dotenv()

from database import *
keep_alive.keep_alive()

description = '''An example bot to showcase the discord.ext.commands extension
module.

There are a number of utility commands being showcased here.'''

intents = discord.Intents.default()
# intents.members = True
intents.message_content = True
intents.members = True

bot = commands.Bot(command_prefix='/', description=description, intents=intents)


@bot.event
async def on_ready():
    print(f'Logged in as {bot.user} (ID: {bot.user.id})')
    print('------')

@bot.command()
async def import_to_linkblocks(ctx, *args):
    channel = str(ctx.channel)
    author = ctx.author
    user_data = retrieve_user_data(author.id).data
    if(len(user_data) == 0):
        await ctx.reply("You are not authenticated with Linkblocks. Please authenticate using /linkblocks_auth.")
        return
    user_data = user_data[0]
    api_key = user_data["api_key"]
    user_id = user_data["user_id"]
    urls = list(args)
    response = requests.post(f'{os.getenv("LINKBLOCKS_URL")}/api/add_bookmark', json={"api_key": api_key, "tag": channel, "user_id": user_id, "urls": urls})
    if response.status_code == 200:
        await ctx.reply("Successfully imported to Linkblocks!")

@bot.command()
async def import_to_linkblocks_with_tag(ctx, tag, *args):
    channel = str(ctx.channel)
    author = ctx.author
    user_data = retrieve_user_data(author.id).data
    if(len(user_data) == 0):
        await ctx.reply("You are not authenticated with Linkblocks. Please authenticate using /linkblocks_auth.")
        return
    user_data = user_data[0]
    api_key = user_data["api_key"]
    user_id = user_data["user_id"]
    urls = list(args)
    response = requests.post(f'{os.getenv("LINKBLOCKS_URL")}/api/add_bookmark', json={"api_key": api_key, "tag": tag, "user_id": user_id, "urls": urls})
    if response.status_code == 200:
        await ctx.reply("Successfully imported to Linkblocks!")

@bot.command()
async def linkblocks_auth(ctx):
    user = discord.utils.get(bot.guilds[0].members, id=ctx.author.id)
    await user.send(f'Please open this link to authenticate: {os.getenv("LINKBLOCKS_URL")}/api/get_key?id={ctx.author.id}')
    await ctx.reply('Please check your DMs for the authentication link.')

bot.run(os.getenv("DISCORD_TOKEN"))