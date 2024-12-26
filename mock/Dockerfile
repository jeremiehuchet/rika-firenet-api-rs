FROM node:22@sha256:0e910f435308c36ea60b4cfd7b80208044d77a074d16b768a81901ce938a62dc
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
