from sock import post


def main():
    # The endpoint and payload you want to send
    endpoint = "/create-didcomm-message"
    payload = {
        "destination_did": "did:nodex:test:EiD9aQYNUJMdgjeQetDj56LNzR6SdwhuXGFalvI3gugPHQ",
        "message": """{"string": "value","number": 1,"boolean": true,"array": [],"map": {}}""",
        "operation_tag": "test-operation-tag",
    }

    # Send the POST request and print the response
    json_response = post(endpoint, payload)
    print(json_response)


if __name__ == "__main__":
    main()
