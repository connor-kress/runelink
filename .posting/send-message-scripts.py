import httpx
from posting import Posting


def on_response(response: httpx.Response, posting: Posting) -> None:
    if not 200 <= response.status_code < 300:
        print(f"Request failed with status {response.status_code}")
        return

    try:
        response_data = response.json() 
        message_id = response_data.get("id")
    except Exception as e:
        posting.notify(f"Error processing response or setting variable: {e}")
        return

    if message_id:
        posting.set_variable("MESSAGE_ID", message_id)
        posting.notify(f"Successfully set MESSAGE_ID to: {message_id}")
        print(f"Successfully set MESSAGE_ID to: {message_id}")
    else:
        posting.notify("Error: 'id' field not found in response JSON.")
        print("Error: 'id' field not found in response JSON.")
        print(f"Response data: {response_data}")
