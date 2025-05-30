import httpx
from posting import Posting


def on_response(response: httpx.Response, posting: Posting) -> None:
    if not 200 <= response.status_code < 300:
        print(f"Request failed with status {response.status_code}")
        return

    try:
        response_data = response.json() 
        user_id = response_data.get("id")
        user_name = response_data.get("name")
        user_domain = response_data.get("domain")
    except Exception as e:
        posting.notify(f"Error processing response or setting variable: {e}")
        return

    if user_id and user_name and user_domain:
        posting.set_variable("USER_ID", user_id)
        posting.set_variable("USER_NAME", user_name)
        posting.set_variable("USER_DOMAIN", user_domain)
        posting.notify(f"Successfully set user to: {user_name}@{user_domain}")
        print(f"Successfully set user to: {user_name}@{user_domain}")
    else:
        posting.notify("Error: valid fields not found in response JSON.")
        print("Error: valid fields not found in response JSON.")
        print(f"Response data: {response_data}")
