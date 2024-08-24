# nodex-agent Installation and Setup Guide

## How to Install by deb package

### Step 1: Download the deb package
Download the deb package from the [release page](https://github.com/nodecross/nodex/releases).

### Step 2: Install the package
Please run the following command. Be sure to replace the filename with the name of the file you downloaded.

```
dpkg -i /path/to/<downloaded_filename>.deb
```

## How to Execute nodex-agent

### Step 1: Set Configuration
Run the following commands to set the configuration. Note that these commands must be executed as the nodex user.

```
sudo -u nodex /usr/bin/nodex-agent --config network set --key project_did --value <your project_did>
sudo -u nodex /usr/bin/nodex-agent --config network set --key secret_key --value <your secret_key>
```

Replace <your project_did> and <your secret_key> with the actual values specific to your project.

### Step 2: Start the nodex-agent
Run the following command to start the nodex-agent.

```
systemctl start nodex-agent
```
