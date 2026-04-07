FROM node:22@sha256:d36ef60bad370a900197ab5168666dab64a03e90c27f4cf818539d826b5be1d3
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
