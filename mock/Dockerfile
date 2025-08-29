FROM node:22@sha256:6fe286835c595e53cdafc4889e9eff903dd3008a3050c1675809148d8e0df805
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
