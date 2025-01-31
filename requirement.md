# Requirements for the Simple Liquidity Pool POC on Substrate

## Objective

The goal of this assignment is to develop a Proof of Concept (POC) for a decentralized liquidity pool using Substrate and Rust. This will assess your expertise in Substrate runtime development, Rust programming, and blockchain UI integration.

## Scope of Work

You will implement a single-asset liquidity pool that allows users to:

- **Add Liquidity**: Users can deposit two tokens (liquidity pair) into the liquidity pool.
- **Withdraw Liquidity**: Users can withdraw their deposited tokens.
- **View Liquidity Balance**: Users can check their liquidity balance through a frontend interface.

## Technical Requirements

### Substrate Pallet Development:

- Implement a custom pallet that supports liquidity deposits and withdrawals.
- Create two custom tokens (e.g., TOKEN_A and TOKEN_B) using the Substrate Assets pallet or a custom pallet.
- Enable users to mint and transfer tokens, as well as deposit these tokens into the liquidity pool.
- Store user liquidity balances efficiently within the pool.
- Emit events for liquidity-related actions (e.g., deposit, withdraw).

### Frontend UI (React + Polkadot.js):

- Develop a simple UI that interacts with the Substrate chain.
- Allow users to:
  - Add liquidity by depositing both TOKEN_A and TOKEN_B, which will create a liquidity pair `{TOKEN_A, TOKEN_B}` in the liquidity pool.
  - Withdraw their liquidity by retrieving their deposited tokens.
  - View their liquidity balance in the pool.

## Documentation

Provide a `README.md` with clear setup instructions, including:

- How to build and run the Substrate node.
- Steps to deploy and interact with the frontend UI.

## Expected Deliverables

- A GitHub repository containing:
  - Substrate runtime code (including pallet implementation).
  - Front-end application code (React + Polkadot.js).
  - Deployment and usage documentation.
  - A functional local test setup demonstrating the working POC.

## Evaluation Criteria

- **Code Quality**: Well-structured and maintainable Rust and TypeScript(UI) code.
- **Blockchain Expertise**: Understanding of Substrate’s storage, extrinsics, and event handling.
- **UI Integration**: The ability to interact with the Substrate node using Polkadot.js or a simple UI built with frontend technologies such as React.
- **Documentation**: Clear, concise, and well-explained setup instructions.

## Timeline

**Total Duration**: 5 Days

## Submission Instructions

1. Fork the provided GitHub repository and work on your implementation in your fork.
2. Once completed, submit a Pull Request (PR) to the original repository with your changes.
3. Along with the PR, include a brief summary covering:
   - Your approach to implementing the POC.
   - Challenges faced and how you addressed them.
   - Potential future improvements you would make if given more time.
