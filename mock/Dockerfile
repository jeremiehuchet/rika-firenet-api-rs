FROM node:22@sha256:6ce3bc6efae14bf65bfe4435d3ba5e9711f97674167a159f49fac2c9b62b2942
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
