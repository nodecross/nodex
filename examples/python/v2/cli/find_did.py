from sock import get

# PLEASE WRITE destination_did
destination_did = (
    "did:nodex:test:DummyDummyDummyDummyDummyDummyDummyDummyDummyD"
)


def main():
    endpoint = "/identifiers/" + destination_did

    json_response = get(endpoint)

    print("The response is as follows.\n")
    print(json_response)


if __name__ == "__main__":
    main()
