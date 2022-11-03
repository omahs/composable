import { ApolloClient, InMemoryCache } from "@apollo/client/core";

export const client = new ApolloClient({
  // uri: process.env.SUBSQUID_URL,
  // TODO: review this URI
  uri: "http://127.0.0.1:4000/graphql",
  cache: new InMemoryCache()
});
