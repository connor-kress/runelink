import httpx
from posting import Posting


def on_response(response: httpx.Response, posting: Posting) -> None:
    if not 200 <= response.status_code < 300:
        print(f"Request failed with status {response.status_code}")
        return

    try:
        response_data = response.json() 
        server_id = response_data.get("id")
    except Exception as e:
        posting.notify(f"Error processing response or setting variable: {e}")
        return

    if server_id:
        posting.set_variable("SERVER_ID", server_id)
        posting.notify(f"Successfully set SERVER_ID to: {server_id}")
        print(f"Successfully set SERVER_ID to: {server_id}")
    else:
        posting.notify("Error: 'id' field not found in response JSON.")
        print("Error: 'id' field not found in response JSON.")
        print(f"Response data: {response_data}")
