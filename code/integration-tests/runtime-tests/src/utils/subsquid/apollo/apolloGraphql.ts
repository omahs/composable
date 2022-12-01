import { ApolloClient, InMemoryCache, HttpLink } from "@apollo/client/core";
import fetch from "cross-fetch";

export const client = new ApolloClient({
  // uri: process.env.SUBSQUID_URL,
  // TODO: review this URI
  uri: "http://127.0.0.1:4000/graphql",
  cache: new InMemoryCache(),
  link: new HttpLink({ uri: "http://127.0.0.1:4000/graphql", fetch })
});
