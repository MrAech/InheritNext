# InheritNext

InheritNext is a decentralized platform built on the Internet Computer designed
to manage digital inheritance. It provides a secure way for users to ensure their
digital assets and data are passed on to their designated heirs according to
their wishes.

## How to run the project

Follow these steps to get your local development environment up and running.

### Prerequisites

You will need to have the IC SDK (dfx) and Node.js installed. Additionally, this
project requires the `candid-extractor` for backend interface generation:

```bash
cargo install candid-extractor
```

### Installation & Setup

1. Clone the repository

```bash

git clone https://github.com/MrAech/InheritNext.git

cd InheritNext

```

1. Start the local replica

Clean any existing state and start the Internet Computer network in the background:

```bash

dfx start --clean --background

```

1. Generate backend declarations

Run the following command to extract the Candid interface for the backend:

```bash

generate-did InheritNext_backend

```

1. Deploy the application

Deploy both the backend and frontend canisters to local network:

```bash

dfx deploy

```

1. Start the frontend

For a fast development experience with hot-reloading, run:

```bash

npm start

```
