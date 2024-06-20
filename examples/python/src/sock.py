import os
import requests_unixsocket
import json
import urllib.parse


# Create a session that can make requests to Unix sockets
session = requests_unixsocket.Session()

# Construct the base URL using the Unix socket path
base = f'http+unix://{urllib.parse.quote(
    os.path.join(os.path.expanduser("~"), ".nodex/run/nodex.sock"), safe="")}'


def call(method, path, payload):
    url = f"{base}:{path}"
    print("Now Requesting...")
    print(f"- Method: {method.upper()}")
    print(f"- URL: {urllib.parse.unquote(url)}\n")

    try:
        if method == "get":
            response = session.get(url)
        elif method == "post":
            response = session.post(url, json=payload)
        else:
            raise ValueError(f"Unsupported method: {method}")
        # Raises stored HTTPError, if one occurred.
        response.raise_for_status()
        return json.dumps(response.json(), indent=4)
    except Exception as e:
        return (
            f"{e.response.status_code} "
            f"{e.response.reason} "
            f"{e.response.text}"
        )


def get(path):
    return call("get", path, None)


def post(path, payload={}):
    return call("post", path, payload)
