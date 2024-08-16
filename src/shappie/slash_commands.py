import nextcord
from nextcord.ext import commands
import os
import dotenv

from bot import ShappieClient

bot = commands.Bot(intents=nextcord.Intents.all(), command_prefix=commands.when_mentioned_or("/"))
