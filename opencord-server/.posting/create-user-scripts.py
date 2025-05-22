import httpx
from posting import Posting


def on_response(response: httpx.Response, posting: Posting) -> None:
    if not 200 <= response.status_code < 300:
        print(f"Request failed with status {response.status_code}")
        return

    try:
        response_data = response.json() 
        user_id = response_data.get("id")
    except Exception as e:
        posting.notify(f"Error processing response or setting variable: {e}")
        return

    if user_id:
        posting.set_variable("USER_ID", user_id)
        posting.notify(f"Successfully set USER_ID to: {user_id}")
        print(f"Successfully set USER_ID to: {user_id}")
    else:
        posting.notify("Error: 'id' field not found in response JSON.")
        print("Error: 'id' field not found in response JSON.")
        print(f"Response data: {response_data}")
