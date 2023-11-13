# Secure Messaging with Node.js Using NodeX Agent

This guide demonstrates the implementation of secure, end-to-end messaging in Node.js applications using NodeX Agent. It focuses on establishing a secure communication channel between two Node.js processes on different machines.

## Overview

Leveraging NodeX Agent, this example illustrates the secure transmission of machine-specific data (CPU temperature) between two processes. It's particularly relevant for developers aiming to incorporate secure messaging in distributed Node.js systems.

This sample serves as a practical guide for developers looking to implement secure messaging in their Node.js applications, particularly useful in scenarios where the data needs to be shared across a network securely.

## Prerequisites

1. **NodeX Agent Setup**: Follow the installation [guide](https://docs.nodecross.io/getting-started/index.html) for NodeX Agent. Post-installation, agents should be visible in NodeX Studio.
2. **Node.js Installation**: Ensure [Node.js](https://nodejs.org/) (version v18.17.0 tested) is installed on your system. Download appropriate versions from here.
3. **Application Code**: Obtain the application using:

```console
curl -L https://github.com/nodecross/nodex/releases/latest/download/example.zip | tar -xz
```

## How to run

In this section, we will guide you through the steps to run this application.

### Step 1. Install the dependencies

Run the following command to install `yarn` as we will use it in our guide.

```console
npm install -g yarn
```

Then, install the dependencies of the application by running the following command:

```console
yarn
```

After the installation, you will be able to run the application.

### Step 2. Run the application

Run the following command to start the application and pass the DID of the other agent as an environment variable.

```console
# In device A
yarn thermometer [DID B]

# In device B
yarn thermometer [DID B]
```

The application will exchange and display the CPU temperatures of both machines.

## Confirm the secure messaging

The application will collect the CPU temperature of the machine and send it to the other agent. And at the same time, it will receive the CPU temperature from the other agent and print it out.

You'll be able to see the messages' logs of the end to end secure messaging on the [log page](https://studio.nodecross.io/logs) of the NodeX Studio.

As a demonstration, this application emits the messages that were sent and received from the other agent.

> **warning**
> Note that you should not log the verified message in your production application.

One is the logs to demonstrate the incoming message from the other agent.

```console
$ yarn thermometer
...
[2023-11-10T21:47:13.062Z] Received: from device did: did:nodex:test:PiCOhgueoa38EZ6E2Vsto5uAfoggHNEM3BRlrSwT3zpxvrp cpu temperature with temperature 27.8 celsius degrees
[2023-11-10T21:47:23.332Z] Received: cpu temperature from device did: did:nodex:test:PiCOhgueoa38EZ6E2Vsto5uAfoggHNEM3BRlrSwT3zpxvrp with temperature 23.8 celsius degrees
...
```

The other is the logs to demonstrate the outgoing message, which the application sends CPU temperature to the other agent.

```console
$ yarn thermometer
...
[2023-11-10T21:53:24.069Z] Sent: with temperature 27.8 celsius degrees to device did: did:nodex:test:PiCOhgueoa38EZ6E2Vsto5uAfoggHNEM3BRlrSwT3zpxvrp
[2023-11-10T21:53:34.340Z] Sent: with temperature 23.8 celsius degrees to device did: did:nodex:test:PiCOhgueoa38EZ6E2Vsto5uAfoggHNEM3BRlrSwT3zpxvrp
... 
```

These message are decrypted and verified by the NodeX Agent and passed to your application.

As can be seen in this example, end-to-end secure communication can be achieved by simply connecting your applications to a local socket provided by the NodeX Agent.

In this case, we only exchanged CPU temperatures, but there are many uses and I believe that all kinds of applications can be built with secure messaging.

## License

This project is licensed under the Apache-2.0 License.
Please see the [LICENSE](../../LICENSE) file for details
