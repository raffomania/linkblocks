import os
from supabase import create_client, Client
from dotenv import load_dotenv
load_dotenv()

url= os.environ.get("SUPABASE_URL")
key = os.environ.get("SUPABASE_KEY")
supabase = create_client(url, key)

def insert_auth_data(discord_id, api_key, user_id):
    if(len(retrieve_user_data(discord_id).data) == 0):
        return supabase.table("auth").insert({"discord_id": discord_id, "api_key": api_key, "user_id": user_id}).execute()
    else:
        return supabase.table("auth").update({"api_key": api_key}).eq("discord_id", discord_id).execute()

def retrieve_user_data(discord_id):
    return supabase.table("auth").select("*").eq("discord_id", discord_id).execute()