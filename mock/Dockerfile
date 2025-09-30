FROM node:22@sha256:d367fd3fce932a9d3bc3816c23f85313d9b44e58e1fed49ef90a10c4b99409e8
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
