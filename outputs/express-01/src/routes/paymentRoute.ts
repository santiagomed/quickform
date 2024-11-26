import express from "express";
import * as paymentController from "../controllers/paymentController";

const router = express.Router();

router.post(
  "/create-checkout-session",
  paymentController.createCheckoutSession
);
router.post("/create-portal-session", paymentController.createPortalSession);
router.post(
  "/webhook",
  express.raw({ type: "application/json" }),
  paymentController.handleWebhook
);

export default router;
