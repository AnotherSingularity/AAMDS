# Relay Integration Guide

To become an authorised relay consumer:

1. Provide a `DestinationId`, a public key, a set of accepted
   classification labels, and community membership.
2. The sponsor operator configures a `DestinationPolicy` and activates
   a new configuration version.
3. Aeon signs and enqueues envelopes; a driver delivers them to your
   endpoint via the sponsor-owned transport.
4. You verify the signature against the destination's public key and
   the anti-replay nonce against your own replay window.

Envelope schema: `../../interface-control-documents/RELAY_SCHEMA.md`.
